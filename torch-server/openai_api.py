"""OpenAI-shaped text-generation API endpoints.

Provides /v1/models, /v1/chat/completions, and /v1/completions.
"""

import logging
import time
import uuid
from typing import Literal

import torch
from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel, ConfigDict, Field, field_validator, model_validator

logger = logging.getLogger(__name__)

router = APIRouter()

MODEL_ID_MAX_CHARS = 256
INPUT_TEXT_MAX_CHARS = 1_000_000
CHAT_MESSAGES_MAX_ITEMS = 256
OUTPUT_TOKENS_MAX_ITEMS = 4_096


# --- Request/Response Models ---


class ClosedRequestModel(BaseModel):
    """Reject input outside the explicitly supported request contract."""

    model_config = ConfigDict(extra="forbid", strict=True)


class ChatMessage(ClosedRequestModel):
    role: Literal["system", "user", "assistant"]
    content: str = Field(min_length=1, max_length=INPUT_TEXT_MAX_CHARS)


class TextGenerationRequest(ClosedRequestModel):
    model: str = Field(min_length=1, max_length=MODEL_ID_MAX_CHARS)
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    top_p: float = Field(default=1.0, gt=0.0, le=1.0)
    max_tokens: int = Field(default=256, ge=1, le=OUTPUT_TOKENS_MAX_ITEMS)
    stream: Literal[False] = False
    stop: None = None

    @field_validator("model")
    @classmethod
    def validate_model_identity(cls, value: str) -> str:
        if value.isspace():
            raise ValueError("model must not be blank")
        return value

    @model_validator(mode="after")
    def validate_sampling_contract(self) -> "TextGenerationRequest":
        if self.temperature == 0.0 and self.top_p != 1.0:
            raise ValueError("top_p must be 1 when temperature is 0")
        return self


class ChatCompletionRequest(TextGenerationRequest):
    messages: list[ChatMessage] = Field(min_length=1, max_length=CHAT_MESSAGES_MAX_ITEMS)

    @model_validator(mode="after")
    def validate_total_message_size(self) -> "ChatCompletionRequest":
        if sum(len(message.content) for message in self.messages) > INPUT_TEXT_MAX_CHARS:
            raise ValueError(f"messages exceed {INPUT_TEXT_MAX_CHARS} characters")
        return self


class CompletionRequest(TextGenerationRequest):
    prompt: str = Field(min_length=1, max_length=INPUT_TEXT_MAX_CHARS)


class AssistantMessage(BaseModel):
    role: Literal["assistant"]
    content: str


class ChatChoice(BaseModel):
    index: int = 0
    message: AssistantMessage
    finish_reason: str = "stop"


class CompletionChoice(BaseModel):
    index: int = 0
    text: str
    finish_reason: str = "stop"


class UsageInfo(BaseModel):
    prompt_tokens: int = 0
    completion_tokens: int = 0
    total_tokens: int = 0


class ChatCompletionResponse(BaseModel):
    id: str
    object: str = "chat.completion"
    created: int
    model: str
    choices: list[ChatChoice]
    usage: UsageInfo = Field(default_factory=UsageInfo)


class CompletionResponse(BaseModel):
    id: str
    object: str = "text_completion"
    created: int
    model: str
    choices: list[CompletionChoice]
    usage: UsageInfo = Field(default_factory=UsageInfo)


# --- Endpoints ---


@router.get("/models")
async def list_models(request: Request):
    """List loaded models in OpenAI format."""
    manager = request.app.state.model_manager
    model_names = manager.list_model_names()

    return {
        "object": "list",
        "data": [
            {
                "id": name,
                "object": "model",
                "created": int(time.time()),
                "owned_by": "local",
            }
            for name in model_names
        ],
    }


@router.post("/chat/completions")
async def chat_completions(req: ChatCompletionRequest, request: Request):
    """Generate one non-streaming text response for a chat prompt."""
    manager = request.app.state.model_manager
    loaded = manager.get_model_for_inference(req.model)

    if loaded is None:
        raise HTTPException(status_code=404, detail=f"Model '{req.model}' not loaded")

    output_text = _generate(loaded, _format_chat_prompt(req.messages), req)

    return ChatCompletionResponse(
        id=f"chatcmpl-{uuid.uuid4().hex[:8]}",
        created=int(time.time()),
        model=req.model,
        choices=[ChatChoice(message=AssistantMessage(role="assistant", content=output_text))],
    )


@router.post("/completions")
async def completions(req: CompletionRequest, request: Request):
    """Generate one non-streaming text completion."""
    manager = request.app.state.model_manager
    loaded = manager.get_model_for_inference(req.model)

    if loaded is None:
        raise HTTPException(status_code=404, detail=f"Model '{req.model}' not loaded")

    output_text = _generate(loaded, req.prompt, req)

    return CompletionResponse(
        id=f"cmpl-{uuid.uuid4().hex[:8]}",
        created=int(time.time()),
        model=req.model,
        choices=[CompletionChoice(text=output_text)],
    )


# --- Generation Helpers ---


def _format_chat_prompt(messages: list[ChatMessage]) -> str:
    """Format chat messages into a single prompt string."""
    parts = []
    for msg in messages:
        if msg.role == "system":
            parts.append(f"System: {msg.content}")
        elif msg.role == "user":
            parts.append(f"User: {msg.content}")
        elif msg.role == "assistant":
            parts.append(f"Assistant: {msg.content}")
    parts.append("Assistant:")
    return "\n".join(parts)


def _generate(loaded, prompt: str, req) -> str:
    """Generate text from a loaded model."""
    tokenizer = loaded.tokenizer
    model = loaded.model
    device = loaded.device

    inputs = tokenizer(prompt, return_tensors="pt").to(device)
    with torch.no_grad():
        outputs = model.generate(
            **inputs,
            max_new_tokens=req.max_tokens,
            temperature=max(req.temperature, 0.01),
            top_p=req.top_p,
            do_sample=req.temperature > 0,
        )

    # Decode only the newly generated tokens
    input_len = inputs["input_ids"].shape[1]
    generated = outputs[0][input_len:]
    return tokenizer.decode(generated, skip_special_tokens=True)

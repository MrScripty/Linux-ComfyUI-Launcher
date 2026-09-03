/**
 * Frontend Error Hierarchy
 *
 * Custom error types for the Pumas Library frontend.
 * Mirrors the backend exception hierarchy from backend/exceptions.py
 */

/**
 * Base error class for all Pumas Library frontend errors.
 */
export class PumasError extends Error {
  constructor(message: string, public override cause?: Error) {
    super(message);
    this.name = this.constructor.name;
    const errorConstructor = Error as typeof Error & {
      captureStackTrace?: (targetObject: object) => void;
    };
    if (typeof errorConstructor.captureStackTrace === 'function') {
      errorConstructor.captureStackTrace(this);
    }
  }
}

/**
 * Network-related errors (HTTP, WebSocket, etc.)
 */
export class NetworkError extends PumasError {
  constructor(
    message: string,
    public url?: string,
    public status?: number,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Desktop bridge API call failures
 */
export class APIError extends PumasError {
  constructor(
    message: string,
    public endpoint?: string,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Input validation failures
 */
export class ValidationError extends PumasError {
  constructor(
    message: string,
    public field?: string,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Metadata corruption or parsing errors
 */
export class MetadataError extends PumasError {
  constructor(
    message: string,
    public filePath?: string,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Process management errors (launch, stop, etc.)
 */
export class ProcessError extends PumasError {
  constructor(
    message: string,
    public exitCode?: number,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Resource management errors (disk space, memory, etc.)
 */
export class ResourceError extends PumasError {
  constructor(
    message: string,
    public resourceType?: string,
    cause?: Error
  ) {
    super(message, cause);
  }
}

/**
 * Type guard helper to check if an error is a known Pumas error.
 */
export function isKnownError(error: unknown): error is PumasError {
  return error instanceof PumasError;
}

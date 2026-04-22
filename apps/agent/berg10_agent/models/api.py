"""REST API endpoints for model configuration management."""

from __future__ import annotations

from typing import Any

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field

from .registry import ModelConfigRegistry
from .schema import ModelConfig


class ModelCreateRequest(BaseModel):
    """Request body for creating/updating a model config."""

    id: str = Field(..., description="Unique model identifier")
    name: str = Field(..., description="Display name")
    model: str = Field(..., description="Provider model string")
    api_key: str = Field(default="", description="Provider API key")
    base_url: str = Field(default="", description="Custom endpoint URL")
    provider: str = Field(default="", description="Provider name")
    is_free: bool = Field(default=False, description="Free tier flag")
    metadata: dict[str, Any] = Field(default_factory=dict)


class ModeDefaultRequest(BaseModel):
    """Request body for setting mode default."""

    model_id: str = Field(..., description="Model ID to set as default")


class ModelResponse(BaseModel):
    """Response for model config."""

    id: str
    name: str
    model: str
    api_key: str
    base_url: str
    provider: str
    is_free: bool
    metadata: dict[str, Any]


class ModeResponse(BaseModel):
    """Response for mode info."""

    mode: str
    default_model_id: str | None


def create_models_router(registry: ModelConfigRegistry) -> APIRouter:
    """Create FastAPI router for model management endpoints."""
    router = APIRouter(prefix="/api/models", tags=["models"])

    @router.get("", response_model=list[ModelResponse])
    async def list_models() -> list[ModelResponse]:
        """List all model configurations."""
        return [ModelResponse(**config.to_dict()) for config in registry.list_all()]

    @router.get("/{model_id}", response_model=ModelResponse)
    async def get_model(model_id: str) -> ModelResponse:
        """Get a specific model configuration."""
        config = registry.get(model_id)
        if not config:
            raise HTTPException(status_code=404, detail=f"Model not found: {model_id}")
        return ModelResponse(**config.to_dict())

    @router.post("", response_model=ModelResponse, status_code=201)
    async def create_model(req: ModelCreateRequest) -> ModelResponse:
        """Create or update a model configuration."""
        config = ModelConfig(
            id=req.id,
            name=req.name,
            model=req.model,
            api_key=req.api_key,
            base_url=req.base_url,
            provider=req.provider,
            is_free=req.is_free,
            metadata=req.metadata,
        )
        registry.upsert(config)
        return ModelResponse(**config.to_dict())

    @router.put("/{model_id}", response_model=ModelResponse)
    async def update_model(model_id: str, req: ModelCreateRequest) -> ModelResponse:
        """Update an existing model configuration."""
        if model_id != req.id:
            raise HTTPException(
                status_code=400,
                detail="Model ID in URL does not match request body",
            )
        config = ModelConfig(
            id=req.id,
            name=req.name,
            model=req.model,
            api_key=req.api_key,
            base_url=req.base_url,
            provider=req.provider,
            is_free=req.is_free,
            metadata=req.metadata,
        )
        registry.upsert(config)
        return ModelResponse(**config.to_dict())

    @router.delete("/{model_id}", status_code=204)
    async def delete_model(model_id: str) -> None:
        """Delete a model configuration."""
        if not registry.delete(model_id):
            raise HTTPException(status_code=404, detail=f"Model not found: {model_id}")

    @router.get("/modes/defaults", response_model=list[ModeResponse])
    async def list_mode_defaults() -> list[ModeResponse]:
        """List all modes and their default models."""
        modes = registry.list_modes()
        return [
            ModeResponse(mode=mode, default_model_id=model_id) for mode, model_id in modes.items()
        ]

    @router.put("/modes/{mode}/default", response_model=ModeResponse)
    async def set_mode_default(mode: str, req: ModeDefaultRequest) -> ModeResponse:
        """Set default model for a mode."""
        try:
            registry.set_mode_default(mode, req.model_id)
        except ValueError as e:
            raise HTTPException(status_code=400, detail=str(e)) from None
        return ModeResponse(mode=mode, default_model_id=req.model_id)

    return router

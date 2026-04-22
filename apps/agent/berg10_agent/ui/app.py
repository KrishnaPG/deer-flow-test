"""Streamlit debug interface connecting to running agent via WebSocket."""

from __future__ import annotations

import json
import math
import threading
from typing import Any

import streamlit as st
import websocket

from berg10_agent.ui.api_client import AgentAPIClient, TestResult
from berg10_agent.ui.catalog import ModelCatalog, ModelInfo

# Constants
MODES = ["build", "plan", "debug", "review"]
ITEMS_PER_PAGE = 25


def run_ui(server_url: str = "ws://localhost:8765/ws") -> None:
    """Launch the Streamlit debug UI."""

    st.set_page_config(page_title="Berg10 Agent Debug", layout="wide")
    st.title("Berg10 Agent Debug UI")

    # Initialize session state
    if "messages" not in st.session_state:
        st.session_state.messages = []
    if "ws_url" not in st.session_state:
        st.session_state.ws_url = server_url
    if "catalog" not in st.session_state:
        st.session_state.catalog = ModelCatalog()
    if "api_url" not in st.session_state:
        # Derive REST API URL from WebSocket URL
        api_url = server_url.replace("ws://", "http://").replace("/ws", "")
        st.session_state.api_url = api_url
    if "api_client" not in st.session_state:
        st.session_state.api_client = AgentAPIClient(st.session_state.api_url)

    # Sidebar configuration
    with st.sidebar:
        st.header("Configuration")
        ws_url = st.text_input("WebSocket URL", value=st.session_state.ws_url)
        st.session_state.ws_url = ws_url

        api_url = st.text_input("REST API URL", value=st.session_state.api_url)
        if api_url != st.session_state.api_url:
            st.session_state.api_url = api_url
            st.session_state.api_client = AgentAPIClient(api_url)

        st.divider()

        # Catalog refresh
        if st.button("🔄 Refresh Catalog"):
            with st.spinner("Fetching model catalog..."):
                st.session_state.catalog.refresh()
            if st.session_state.catalog.error:
                st.error(f"Catalog error: {st.session_state.catalog.error}")
            else:
                st.success(f"Loaded {len(st.session_state.catalog.get_cached())} models")
            st.rerun()

        if st.session_state.catalog.is_cached:
            st.caption(f"Catalog: {len(st.session_state.catalog.get_cached())} models")

        st.divider()

        # Navigation
        tab = st.radio("Navigation", ["Chat", "Models", "Modes"])

    # Main content based on selected tab
    if tab == "Chat":
        _render_chat_tab()
    elif tab == "Models":
        _render_models_tab()
    elif tab == "Modes":
        _render_modes_tab()


def _render_chat_tab() -> None:
    """Render the chat interface with model/mode selectors."""
    # Model and mode selectors
    col1, col2 = st.columns(2)
    with col1:
        # Get configured models from API
        try:
            models = st.session_state.api_client.list_models()
            model_options = {m["id"]: f"{m['name']} ({m['id']})" for m in models}
            model_options[""] = "Select a model..."
        except Exception:
            model_options = {"": "⚠️ Cannot connect to API"}

        selected_model = st.selectbox(
            "Model",
            options=list(model_options.keys()),
            format_func=lambda x: model_options.get(x, x),
            key="chat_model_select",
        )

    with col2:
        selected_mode = st.selectbox("Mode", options=MODES, key="chat_mode_select")

    st.divider()

    # Chat display
    for msg in st.session_state.messages:
        with st.chat_message(msg["role"]):
            st.markdown(msg["content"])
            if "model_id" in msg:
                st.caption(f"Model: {msg['model_id']}")

    # Input
    if prompt := st.chat_input("Send a message to the agent"):
        st.session_state.messages.append({"role": "user", "content": prompt})

        with st.chat_message("user"):
            st.markdown(prompt)

        with st.chat_message("assistant"):
            placeholder = st.empty()
            full_response, model_id = _send_message(
                st.session_state.ws_url,
                prompt,
                model_id=selected_model if selected_model else None,
                mode=selected_mode,
            )
            placeholder.markdown(full_response)

        st.session_state.messages.append(
            {
                "role": "assistant",
                "content": full_response,
                "model_id": model_id,
            }
        )

    if st.button("Clear History"):
        st.session_state.messages = []
        st.rerun()


def _render_models_tab() -> None:
    """Render the model configuration interface."""
    tab_browse, tab_mine = st.tabs(["Browse Catalog", "My Models"])

    with tab_browse:
        _render_catalog_browser()

    with tab_mine:
        _render_my_models()


def _render_catalog_browser() -> None:
    """Render the model catalog browser."""
    catalog = st.session_state.catalog

    if not catalog.is_cached:
        st.info("Click '🔄 Refresh Catalog' in the sidebar to load available models.")
        return

    # Get existing model IDs to disable add button
    api = st.session_state.api_client
    try:
        existing_models = api.list_models()
        existing_ids = {m["model"] for m in existing_models}
    except Exception:
        existing_ids = set()

    # Search and filters
    col1, col2, col3, col4 = st.columns([3, 1, 1, 1])
    with col1:
        search_query = st.text_input("🔍 Search models", placeholder="Type to search...")
    with col2:
        filter_free = st.checkbox("Free only", value=False)
    with col3:
        filter_fc = st.checkbox("Function calling")
    with col4:
        filter_vision = st.checkbox("Vision")

    # Provider filter
    providers = catalog.get_providers()
    selected_provider = st.selectbox("Provider", ["All"] + providers)

    # Apply search and filters
    results = catalog.search(search_query) if search_query else catalog.get_cached()

    results = catalog.filter_models(
        results,
        free_only=filter_free,
        function_calling=filter_fc,
        vision=filter_vision,
        provider=selected_provider if selected_provider != "All" else "",
    )

    st.caption(f"Found {len(results)} models")

    # Pagination
    total_pages = max(1, math.ceil(len(results) / ITEMS_PER_PAGE))
    if "catalog_page" not in st.session_state:
        st.session_state.catalog_page = 0

    col_prev, col_page, col_next = st.columns([1, 2, 1])
    with col_prev:
        if st.button("◀ Prev", disabled=st.session_state.catalog_page == 0):
            st.session_state.catalog_page -= 1
            st.rerun()
    with col_page:
        st.markdown(f"**Page {st.session_state.catalog_page + 1} of {total_pages}**")
    with col_next:
        if st.button("Next ▶", disabled=st.session_state.catalog_page >= total_pages - 1):
            st.session_state.catalog_page += 1
            st.rerun()

    # Display current page
    start = st.session_state.catalog_page * ITEMS_PER_PAGE
    end = min(start + ITEMS_PER_PAGE, len(results))
    page_results = results[start:end]

    for model in page_results:
        with st.container():
            col1, col2, col3, col4 = st.columns([4, 2, 2, 1])
            with col1:
                st.markdown(f"**{model.name}**")
                st.caption(f"`{model.id}`")
                if model.description:
                    st.caption(model.description[:100])
            with col2:
                st.caption(f"Provider: {model.provider}")
                if model.is_free:
                    st.badge("Free", color="primary")
            with col3:
                cost = model.input_cost_per_token + model.output_cost_per_token
                st.caption(f"Cost: ${cost * 1000:.4f}/1K tokens")
                ctx = model.max_input_tokens or model.context_length
                if ctx:
                    st.caption(f"Context: {ctx:,} tokens")
            with col4:
                caps = []
                if model.supports_function_calling:
                    caps.append("🔧")
                if model.supports_vision:
                    caps.append("👁️")
                if model.supports_reasoning:
                    caps.append("🧠")
                if caps:
                    st.markdown(" ".join(caps))

                model_id = model.id
                is_added = model_id in existing_ids
                label = "Added ✓" if is_added else "Add"
                if st.button(label, key=f"add_{model.id}", disabled=is_added):
                    _add_model_from_catalog(model)

            st.divider()


def _render_my_models() -> None:
    """Render the user's configured models."""
    api = st.session_state.api_client

    # Add new model form
    with st.expander("➕ Add New Model", expanded=False):
        _render_model_form()

    st.divider()

    # List existing models
    try:
        models = api.list_models()
    except Exception as e:
        st.error(f"Cannot load models: {e}")
        return

    if not models:
        st.info("No models configured yet. Add one above or browse the catalog.")
        return

    st.subheader("Configured Models")

    for model in models:
        with st.container():
            col1, col2, col3 = st.columns([3, 2, 2])
            with col1:
                st.markdown(f"**{model['name']}**")
                st.caption(f"ID: `{model['id']}` | Model: `{model['model']}`")
            with col2:
                st.caption(f"Provider: {model.get('provider', 'N/A')}")
                if model.get("is_free"):
                    st.badge("Free", color="primary")
            with col3:
                btn_col1, btn_col2, btn_col3 = st.columns(3)
                with btn_col1:
                    if st.button("✏️ Edit", key=f"edit_{model['id']}"):
                        st.session_state[f"editing_{model['id']}"] = True
                        st.rerun()
                with btn_col2:
                    if st.button("🧪 Test", key=f"test_{model['id']}"):
                        st.session_state[f"testing_{model['id']}"] = True
                        st.rerun()
                with btn_col3:
                    if st.button("🗑️ Delete", key=f"delete_{model['id']}"):
                        api.delete_model(model["id"])
                        st.rerun()

            # Edit form (shown when edit button clicked)
            if st.session_state.get(f"editing_{model['id']}"):
                with st.expander("Edit Model", expanded=True):
                    _render_model_form(existing=model)
                    if st.button("Cancel Edit", key=f"cancel_{model['id']}"):
                        del st.session_state[f"editing_{model['id']}"]
                        st.rerun()

            # Test results (shown when test button clicked)
            if st.session_state.get(f"testing_{model['id']}"):
                with st.expander("Test Connection", expanded=True):
                    _render_test_result(model)
                    if st.button("Close", key=f"close_test_{model['id']}"):
                        del st.session_state[f"testing_{model['id']}"]
                        st.rerun()

            st.divider()


def _render_model_form(existing: dict[str, Any] | None = None) -> None:
    """Render the add/edit model form."""
    api = st.session_state.api_client
    is_edit = existing is not None

    with st.form(key=f"model_form_{existing['id'] if is_edit else 'new'}"):
        model_id = st.text_input(
            "Model ID",
            value=existing["id"] if is_edit else "",
            disabled=is_edit,
            help="Unique identifier for this model config",
        )
        name = st.text_input(
            "Display Name",
            value=existing["name"] if is_edit else "",
            help="Human-readable name",
        )
        model_str = st.text_input(
            "Model String",
            value=existing["model"] if is_edit else "",
            help="e.g., openrouter/google/gemma-3-12b-it",
        )

        # API Key with show/hide
        api_key_col1, api_key_col2 = st.columns([4, 1])
        with api_key_col1:
            show_key = st.checkbox("Show API Key", key=f"show_key_{model_id}")
            input_type = "default" if show_key else "password"
            api_key = st.text_input(
                "API Key",
                value=existing.get("api_key", "") if is_edit else "",
                type=input_type,
                key=f"api_key_{model_id}",
            )
        with api_key_col2:
            st.write("")  # Spacer
            st.write("")  # Spacer
            st.caption("Provider-specific key")

        base_url = st.text_input(
            "Base URL (optional)",
            value=existing.get("base_url", "") if is_edit else "",
            help="Custom API endpoint URL",
        )
        provider = st.text_input(
            "Provider",
            value=existing.get("provider", "") if is_edit else "",
            help="e.g., openrouter, openai, anthropic",
        )
        is_free = st.checkbox(
            "Free tier model",
            value=existing.get("is_free", False) if is_edit else False,
        )

        submitted = st.form_submit_button("Save Model")

        if submitted:
            if not model_id or not name or not model_str:
                st.error("Model ID, Name, and Model String are required.")
                return

            from berg10_agent.models import ModelConfig

            config = ModelConfig(
                id=model_id,
                name=name,
                model=model_str,
                api_key=api_key,
                base_url=base_url,
                provider=provider,
                is_free=is_free,
            )

            try:
                if is_edit:
                    api.update_model(config)
                    del st.session_state[f"editing_{model_id}"]
                else:
                    api.create_model(config)
                st.success(f"Model '{name}' saved!")
                st.rerun()
            except Exception as e:
                st.error(f"Failed to save: {e}")


def _add_model_from_catalog(catalog_model: ModelInfo) -> None:
    """Add a model from the catalog to the user's configured models."""
    api = st.session_state.api_client

    from berg10_agent.models import ModelConfig

    config = ModelConfig(
        id=catalog_model.id.replace("/", "-").replace(":", "-"),
        name=catalog_model.name,
        model=catalog_model.id,
        api_key="",  # User needs to provide this
        provider=catalog_model.provider,
        is_free=catalog_model.is_free,
    )

    try:
        api.create_model(config)
        st.success(f"Added '{catalog_model.name}' to My Models! Don't forget to add your API key.")
        st.rerun()
    except Exception as e:
        st.error(f"Failed to add model: {e}")


def _render_test_result(model_config: dict[str, Any]) -> None:
    """Render detailed test connection results."""
    api = st.session_state.api_client

    # Get catalog info for capabilities
    catalog = st.session_state.catalog
    catalog_info = None
    if catalog.is_cached:
        for m in catalog.get_cached():
            if m.id == model_config["model"] or m.id == model_config["id"]:
                catalog_info = m
                break

    capabilities = {}
    if catalog_info:
        capabilities = {
            "Function Calling": catalog_info.supports_function_calling,
            "Vision": catalog_info.supports_vision,
            "System Messages": catalog_info.supports_system_messages,
            "Reasoning": catalog_info.supports_reasoning,
            "Prompt Caching": catalog_info.supports_prompt_caching,
            "Response Schema": catalog_info.supports_response_schema,
        }

    # Run test in a thread
    with st.spinner("Testing connection..."):
        from berg10_agent.models import ModelConfig

        config = ModelConfig(
            id=model_config["id"],
            name=model_config["name"],
            model=model_config["model"],
            api_key=model_config.get("api_key", ""),
            base_url=model_config.get("base_url", ""),
            provider=model_config.get("provider", ""),
        )

        # Run sync test in thread to not block UI
        result_container: list[TestResult] = []

        def run_test() -> None:
            result = api.test_model_connection(config, timeout=30, capabilities=capabilities)
            result_container.append(result)

        thread = threading.Thread(target=run_test)
        thread.start()
        thread.join(timeout=35)  # Slightly more than 30s timeout

        if thread.is_alive():
            st.error("Test timed out after 30 seconds")
            return

        if not result_container:
            st.error("Test failed to complete")
            return

        result = result_container[0]

    # Display results
    if result.success:
        st.success("✅ Connection successful!")
    else:
        st.error("❌ Connection failed!")

    col1, col2 = st.columns(2)
    with col1:
        st.metric("Latency", f"{result.latency_ms:,.0f} ms")
        st.metric("Model", result.model)
    with col2:
        st.metric("Input Tokens", result.input_tokens)
        st.metric("Output Tokens", result.output_tokens)

    if result.response_content:
        st.text_area("Response", value=result.response_content, height=60, disabled=True)

    if result.error:
        st.error(f"**Error:** {result.error}")

    if capabilities:
        st.subheader("Capabilities")
        cap_cols = st.columns(3)
        for i, (name, supported) in enumerate(capabilities.items()):
            with cap_cols[i % 3]:
                icon = "✅" if supported else "❌"
                st.markdown(f"{icon} {name}")


def _render_modes_tab() -> None:
    """Render mode default configuration."""
    api = st.session_state.api_client

    st.subheader("Mode Default Models")
    st.caption("Configure which model to use for each agent mode.")

    try:
        current_defaults = api.list_mode_defaults()
        models = api.list_models()
        model_options = {m["id"]: m["name"] for m in models}
    except Exception as e:
        st.error(f"Cannot load configuration: {e}")
        return

    if not models:
        st.warning("No models configured. Add models in the Models tab first.")
        return

    # Mode configuration
    mode_configs: dict[str, str | None] = {}
    for mode in MODES:
        current = current_defaults.get(mode)
        options = list(model_options.keys())
        # Add empty option
        options_with_none = [None] + options

        selected = st.selectbox(
            f"{mode.title()} Mode",
            options=options_with_none,
            format_func=lambda x: "None (not set)" if x is None else f"{model_options[x]} ({x})",
            index=options_with_none.index(current) if current in options_with_none else 0,
            key=f"mode_{mode}",
        )
        mode_configs[mode] = selected

    st.divider()

    if st.button("Save All Mode Defaults"):
        success_count = 0
        error_msgs = []
        for mode, model_id in mode_configs.items():
            if model_id:
                try:
                    api.set_mode_default(mode, model_id)
                    success_count += 1
                except Exception as e:
                    error_msgs.append(f"{mode}: {e}")

        if error_msgs:
            st.error(f"Some modes failed to save: {'; '.join(error_msgs)}")
        if success_count:
            st.success(f"Saved {success_count} mode default(s)!")
            st.rerun()


def _send_message(
    ws_url: str,
    content: str,
    model_id: str | None = None,
    mode: str = "build",
) -> tuple[str, str]:
    """Send message via WebSocket and collect response."""
    try:
        ws = websocket.create_connection(ws_url)
        msg = {
            "type": "message",
            "content": content,
            "mode": mode,
        }
        if model_id:
            msg["model_id"] = model_id

        ws.send(json.dumps(msg))

        responses: list[str] = []
        used_model = model_id or ""

        while True:
            raw = ws.recv()
            data = json.loads(raw)
            msg_type = data.get("type", "")

            if msg_type == "done":
                result = data.get("result", {})
                if isinstance(result, dict):
                    responses.append(result.get("answer", ""))
                break
            elif msg_type == "error":
                responses.append(f"Error: {data.get('message', 'Unknown error')}")
                break
            elif msg_type == "chunk":
                responses.append(data.get("content", ""))

        ws.close()
        return "".join(responses), used_model

    except Exception as e:
        return f"Connection error: {e}", model_id or ""


if __name__ == "__main__":
    run_ui()

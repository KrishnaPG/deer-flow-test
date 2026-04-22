"""Streamlit debug interface connecting to running agent via WebSocket."""

from __future__ import annotations

import json
from typing import Any


def run_ui(server_url: str = "ws://localhost:8765/ws") -> None:
    """Launch the Streamlit debug UI."""
    import streamlit as st

    st.set_page_config(page_title="Berg10 Agent Debug", layout="wide")
    st.title("Berg10 Agent Debug UI")

    if "messages" not in st.session_state:
        st.session_state.messages = []
    if "ws_url" not in st.session_state:
        st.session_state.ws_url = server_url

    # Sidebar config
    with st.sidebar:
        st.header("Configuration")
        ws_url = st.text_input("WebSocket URL", value=st.session_state.ws_url)
        st.session_state.ws_url = ws_url

        if st.button("Clear History"):
            st.session_state.messages = []
            st.rerun()

    # Chat display
    for msg in st.session_state.messages:
        with st.chat_message(msg["role"]):
            st.markdown(msg["content"])

    # Input
    if prompt := st.chat_input("Send a message to the agent"):
        st.session_state.messages.append({"role": "user", "content": prompt})

        with st.chat_message("user"):
            st.markdown(prompt)

        with st.chat_message("assistant"):
            placeholder = st.empty()
            full_response = _send_message(st.session_state.ws_url, prompt)
            placeholder.markdown(full_response)

        st.session_state.messages.append({"role": "assistant", "content": full_response})


def _send_message(ws_url: str, content: str) -> str:
    """Send message via WebSocket and collect response."""
    try:
        import websocket

        ws = websocket.create_connection(ws_url)
        ws.send(json.dumps({"type": "message", "content": content}))

        responses: list[str] = []
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
        return "".join(responses)

    except Exception as e:
        return f"Connection error: {e}"


if __name__ == "__main__":
    run_ui()

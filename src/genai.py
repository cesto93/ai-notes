from typing import TypedDict
from pydantic import BaseModel, Field
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.language_models.chat_models import BaseChatModel
from langchain.chat_models import init_chat_model
from langgraph.graph import StateGraph, START, END
from src.storage import save_note


class NoteMetadata(BaseModel):
    """Data model for note metadata extracted from text."""

    Title: str = Field(
        ...,
        description="The title of the note. This should be a concise word categorizing the note",
    )
    Directory: str = Field(
        ...,
        description="The directory where the note should be stored. This should be a single word representing the category of the note (e.g. 'work', 'personal', 'ideas', 'learning').",
    )


class State(TypedDict):
    note: str
    metadata: NoteMetadata
    llm: BaseChatModel
    action: str


def create_agent():
    """
    Creates a Google Generative AI agent with the specified model.

    Args:
        model (str): The model to use for the agent.

    Returns:
        ChatGoogleGenerativeAI: An instance of the Google Generative AI agent.
    """

    graph_builder = StateGraph(State)
    graph_builder.add_node("summarize_text", summarize_text)
    graph_builder.add_node("paraphrase_text", paraphrase_text)
    graph_builder.add_node("extract_metadata", extract_metadata)
    graph_builder.add_node("save_note_action", save_note_action)

    def route_action(state: State):
        return state.get("action", "paraphrase")

    graph_builder.add_conditional_edges(
        START,
        route_action,
        {
            "summarize": "summarize_text",
            "paraphrase": "paraphrase_text",
            "paraphrase_view": "paraphrase_text",
        }
    )

    def route_after_paraphrase(state: State):
        if state.get("action") == "paraphrase_view":
            return "end"
        return "extract_metadata"

    graph_builder.add_edge("summarize_text", END)
    graph_builder.add_conditional_edges(
        "paraphrase_text",
        route_after_paraphrase,
        {
            "end": END,
            "extract_metadata": "extract_metadata"
        }
    )
    graph_builder.add_edge("extract_metadata", "save_note_action")

    graph_builder.add_edge("save_note_action", END)
    graph = graph_builder.compile()

    return graph


def get_initial_state(model: str, note: str, action: str) -> State:
    """
    Returns the initial state for the agent.

    Returns:
        State: The initial state containing default values.
    """
    llm = init_chat_model(f"google_genai:{model}")
    return {
        "note": note,
        "metadata": NoteMetadata(Title="", Directory=""),
        "llm": llm,
        "action": action,
    }


def summarize_text(state: State):
    """
    Summarizes the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for summarization.
        text (str): The text to summarize.

    Returns:
        str: The summarized text.
    """
    text = state["note"]
    prompt_template = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Summarize the user note in a concise manner.",
            ),
            ("human", "{text}"),
        ]
    )

    prompt = prompt_template.invoke({"text": text})
    llm = state["llm"]
    result = llm.invoke(prompt)
    return {"note": result.content}


def paraphrase_text(state: State):
    """
    Paraphrases the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for summarization.
        text (str): The text to summarize.

    Returns:
        str: The summarized text.
    """

    text = state["note"]
    prompt_template = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Paraphrase the user note making it clearer.",
            ),
            ("human", "{text}"),
        ]
    )

    prompt = prompt_template.invoke({"text": text})
    llm = state["llm"]
    result = llm.invoke(prompt)
    return {"note": result.content}


def extract_metadata(state: State):
    """
    Extracts metadata from the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for metadata extraction.
        text (str): The text from which to extract metadata.

    Returns:
        NoteMetadata: An instance containing the extracted metadata.
    """
    text = state["note"]
    prompt_template = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "You are an expert extraction algorithm. "
                "Only extract relevant information from the text. "
                "If you do not know the value of an attribute asked to extract, "
                "return null for the attribute's value.",
            ),
            ("human", "{text}"),
        ]
    )

    llm = state["llm"]
    structured_llm = llm.with_structured_output(schema=NoteMetadata)
    prompt = prompt_template.invoke({"text": text})
    result = structured_llm.invoke(prompt)
    if isinstance(result, NoteMetadata):
        return {"metadata": result}
    else:
        raise ValueError("The result is not of type NoteMetadata.")


def save_note_action(state: State):
    """
    Saves the note and its metadata to the storage.

    Args:
        state (State): The state containing the note and metadata.
    """

    note = state["note"]
    metadata = state["metadata"]
    save_note(note, metadata.Title, metadata.Directory)

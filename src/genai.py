from langchain_google_genai import ChatGoogleGenerativeAI
from pydantic import BaseModel, Field
from langchain_core.prompts import ChatPromptTemplate


class NoteMetadata(BaseModel):
    """Data model for note metadata extracted from text."""
    Argument: str = Field(..., description="The main argument or point of the note.")
    Tags: list[str] = Field(..., description="List of tags associated with the note.")


def create_agent(model: str) -> ChatGoogleGenerativeAI:
    """
    Creates a Google Generative AI agent with the specified model.

    Args:
        model (str): The model to use for the agent.

    Returns:
        ChatGoogleGenerativeAI: An instance of the Google Generative AI agent.
    """
    return ChatGoogleGenerativeAI(model=model)


def summarize_text(llm: ChatGoogleGenerativeAI, text: str) -> str:
    """
    Summarizes the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for summarization.
        text (str): The text to summarize.

    Returns:
        str: The summarized text.
    """
    if not text.strip():
        raise ValueError("Text to summarize cannot be empty.")

    result = llm.invoke(f"Summarize this note: {text}")
    return result.content


def extract_metadata(llm: ChatGoogleGenerativeAI, text: str) -> NoteMetadata:
    """
    Extracts metadata from the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for metadata extraction.
        text (str): The text from which to extract metadata.

    Returns:
        NoteMetadata: An instance containing the extracted metadata.
    """
    if not text.strip():
        raise ValueError("Text to extract metadata from cannot be empty.")

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

    structured_llm = llm.with_structured_output(schema=NoteMetadata)
    prompt = prompt_template.invoke({"text": text})
    result = structured_llm.invoke(prompt)
    if isinstance(result, NoteMetadata):
        return result
    else:
        raise ValueError("The result is not of type NoteMetadata.")

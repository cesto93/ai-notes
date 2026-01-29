from pydantic import BaseModel, Field
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.language_models.chat_models import BaseChatModel
from langchain.chat_models import init_chat_model


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


def get_model(provider: str, model: str) -> BaseChatModel:
    """
    Returns the chat model based on provider and model name.
    """
    if provider == "google":
        return init_chat_model(model, model_provider="google_genai")
    elif provider == "ollama":
        return init_chat_model(model, model_provider="ollama")
    elif provider == "groq":
        return init_chat_model(model, model_provider="groq")
    else:
        return init_chat_model(model, model_provider="google_genai")


def summarize_text(llm: BaseChatModel, text: str) -> str:
    """
    Summarizes the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for summarization.
        text (str): The text to summarize.

    Returns:
        str: The summarized text.
    """
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
    result = llm.invoke(prompt)
    return result.content


def paraphrase_text(llm: BaseChatModel, text: str) -> str:
    """
    Paraphrases the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for summarization.
        text (str): The text to summarize.

    Returns:
        str: The summarized text.
    """

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
    result = llm.invoke(prompt)
    return result.content


def extract_metadata(llm: BaseChatModel, text: str) -> NoteMetadata:
    """
    Extracts metadata from the given text using the provided language model.

    Args:
        llm (ChatGoogleGenerativeAI): The language model to use for metadata extraction.
        text (str): The text from which to extract metadata.

    Returns:
        NoteMetadata: An instance containing the extracted metadata.
    """
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

from pydantic import BaseModel, Field
from langchain_core.prompts import ChatPromptTemplate
from langchain_core.language_models.chat_models import BaseChatModel
from langchain.chat_models import init_chat_model


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


def generate_mindmap(llm: BaseChatModel, text: str) -> str:
    """
    Generates a Mermaid mindmap from the given text using the provided language model.

    Args:
        llm (BaseChatModel): The language model to use for generation.
        text (str): The text to transform into a mindmap.

    Returns:
        str: The Mermaid mindmap syntax.
    """
    prompt_template = ChatPromptTemplate.from_messages(
        [
            (
                "system",
                "Create a Mermaid.js mindmap syntax for the following text. "
                "Only return the Mermaid syntax starting with 'mindmap' and nothing else. "
                "Do not use markdown code blocks. "
                "Ensure the syntax is valid for Mermaid.js.",
            ),
            ("human", "{text}"),
        ]
    )

    prompt = prompt_template.invoke({"text": text})
    result = llm.invoke(prompt)
    return result.content

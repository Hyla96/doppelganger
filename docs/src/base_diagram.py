"""
Abstract base class for diagram generators.
"""

from abc import ABC, abstractmethod


class BaseDiagram(ABC):
    """Abstract base class for all diagram generators."""

    @property
    @abstractmethod
    def name(self) -> str:
        """The display name of the diagram."""
        pass

    @property
    @abstractmethod
    def file_name(self) -> str:
        """The filename (without extension) for the generated diagram."""
        pass

    @abstractmethod
    def generate(self) -> None:
        """Generate the diagram content. This method should contain the diagram definition."""
        pass

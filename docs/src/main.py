"""
Main script to generate all diagrams.
Run with: uv run src/main.py
"""

from diagram_generators.high_level_architecture import HighLevelArchitectureDiagram
from diagrams import Diagram
from pathlib import Path
import sys

src_path = Path(__file__).parent
sys.path.insert(0, str(src_path))

OUTPUT_DIRECTORY = "diagrams"
FILE_EXTENTION = "jpg"


_GENERATORS = [
    HighLevelArchitectureDiagram(),
]


def main():
    """Generate all diagrams."""
    diagrams_dir = Path(OUTPUT_DIRECTORY)
    diagrams_dir.mkdir(exist_ok=True)

    print("Generating diagrams...")

    # List of all diagram generation functions

    # Generate each diagram
    for diagram in _GENERATORS:
        path = f"{OUTPUT_DIRECTORY}/{diagram.file_name}"
        try:
            print(f"  • Generating {diagram.name}...")

            with Diagram(
                name=diagram.name,
                show=False,
                outformat=FILE_EXTENTION,
                filename=path,
            ):
                diagram.generate()
            print(
                f'    ✓ {diagram.name} generated successfully at "{path}.{FILE_EXTENTION}"'
            )
        except Exception as e:
            print(
                f'    ✗ Error generating {diagram.name} at "{path}.{FILE_EXTENTION}": {e}'
            )

    print("\nAll diagrams generated!")


if __name__ == "__main__":
    main()

"""
Auto-generated Python stubs for nucleation.
A high-performance Minecraft schematic parser and utility library.

This file is generated from api_definition.rs - DO NOT EDIT MANUALLY!
"""

from typing import Optional, List, Dict, Union, Any

class BlockState:
    """Represents a Minecraft block with its properties"""

    def __init__(self, name: str) -> None:
        """Create a new BlockState with the given name"""
        ...

    @property
    def name(self) -> str: 
        """The block's resource name (e.g., 'minecraft:stone')"""
        ...

    @property
    def properties(self) -> Dict[str, str]: 
        """The block's state properties"""
        ...

    def with_property(self, key: str, value: str) -> BlockState:
        """Create a new BlockState with an additional property"""
        ...


class Schematic:
    """A Minecraft schematic containing blocks and metadata"""

    def __init__(self, name: Optional[str] = Default) -> None:
        """Create a new empty schematic"""
        ...

    @property
    def dimensions(self) -> List[int]: 
        """The schematic's dimensions [width, height, length]"""
        ...

    @property
    def block_count(self) -> int: 
        """Total number of non-air blocks"""
        ...

    @property
    def volume(self) -> int: 
        """Total volume of the schematic"""
        ...

    @property
    def region_names(self) -> List[str]: 
        """Names of all regions in the schematic"""
        ...

    def from_data(self, data: bytes) -> None:
        """Load schematic from byte data, auto-detecting format"""
        ...

    def from_litematic(self, data: bytes) -> None:
        """Load schematic from Litematic format data"""
        ...

    def to_litematic(self) -> bytes:
        """Convert schematic to Litematic format"""
        ...

    def from_schematic(self, data: bytes) -> None:
        """Load schematic from classic .schematic format data"""
        ...

    def to_schematic(self) -> bytes:
        """Convert schematic to classic .schematic format"""
        ...

    def set_block(self, x: int, y: int, z: int, block_name: str) -> None:
        """Set a block at the specified position"""
        ...

    def set_block_with_properties(self, x: int, y: int, z: int, block_name: str, properties: Dict[str, str]) -> None:
        """Set a block at the specified position with properties"""
        ...

    def get_block(self, x: int, y: int, z: int) -> Optional[BlockState]:
        """Get block at the specified position"""
        ...


def load_schematic(path: str) -> Schematic:
    """Load a schematic from file path"""
    ...

def save_schematic(schematic: Schematic, path: str, format: str = auto) -> None:
    """Save a schematic to file path"""
    ...

def debug_schematic(schematic: Schematic) -> str:
    """Get debug information for a schematic"""
    ...

def debug_json_schematic(schematic: Schematic) -> str:
    """Get debug information for a schematic in JSON format"""
    ...


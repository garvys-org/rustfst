from typing import Optional


class DrawingConfig:
    def __init__(
        self,
        acceptor: bool = False,
        title: str = "",
        width: Optional[float] = None,
        height: Optional[float] = None,
        portrait: bool = True,
        vertical: bool = False,
        ranksep: Optional[float] = None,
        nodesep: Optional[float] = None,
        fontsize: int = 14,
        show_weight_one: bool = True,
        print_weight: bool = True,
    ):
        """
        Args:
            acceptor: Should the figure be rendered in acceptor format if possible?
            title: An optional string indicating the figure title.
            width: The figure width, in inches.
            height: The figure height, in inches.
            portrait: Should the figure be rendered in portrait rather than
                landscape?
            vertical: Should the figure be rendered bottom-to-top rather than
                left-to-right?
            ranksep: The minimum separation separation between ranks, in inches.
            nodesep: The minimum separation between nodes, in inches.
            fontsize: Font size, in points.
            show_weight_one: Should weights equivalent to semiring One be printed?
            print_weight: Should weights be printed
        """
        self.acceptor = acceptor
        self.title = title
        self.width = width
        self.height = height
        self.portrait = portrait
        self.vertical = vertical
        self.ranksep = ranksep
        self.nodesep = nodesep
        self.fontsize = fontsize
        self.show_weight_one = show_weight_one
        self.print_weight = print_weight

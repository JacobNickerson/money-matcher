from PyQt5.QtWidgets import ( 
    QWidget, QVBoxLayout, QHBoxLayout, QPushButton, QLabel, QLineEdit,
    QSizePolicy, QTableView, QStyledItemDelegate, QTabBar, QHeaderView,
    QFrame, QGridLayout, QComboBox, QScrollArea
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QLinearGradient, QIcon
)
from PyQt5.QtCore import (
    Qt, QRect, QSize
)
from PyQt5.Qsci import (
    QsciScintilla, QsciLexerPython
)

class Header(QWidget):
    def __init__(self):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            Header {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(12)

        title = QLabel("Strategies")
        title.setFont(QFont("Inter", 20, QFont.Bold))
        title.setStyleSheet("color: #FFFFFF; background: transparent;")
        layout.addWidget(title)

        spacer = QWidget()
        spacer.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        spacer.setStyleSheet("background-color: transparent")
        layout.addWidget(spacer)

        combo_style = """
            QComboBox {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 8px;
                padding: 8px;
                color: white;
            }
            QComboBox::drop-down {
                border: none;
                subcontrol-origin: padding;
                subcontrol-position: top right;
                width: 20px;
            }
            QComboBox::down-arrow {
                image: url(../../resources/images/down-arrow.svg);
                width: 16px;
                height: 16px;
                margin-right: 16px;
            }
            QComboBox QAbstractItemView {
                background-color: #080808;
                selection-background-color: #363636;
                color: white;
            }
        """
        
        self.symbol_list = QComboBox()
        self.symbol_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.symbol_list.setStyleSheet(combo_style)
        self.symbol_list.addItems(["SOL/USD", "BTC/USD", "ETH/USD"])
        self.symbol_list.setMinimumWidth(200)
        layout.addWidget(self.symbol_list)

        self.strategy_list = QComboBox()
        self.strategy_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.strategy_list.setStyleSheet(combo_style)
        self.strategy_list.addItems(["Metatron", "Momentum", "Arbitrage", "Scalping"])
        self.strategy_list.setMinimumWidth(400)
        layout.addWidget(self.strategy_list)

        btn_container = QWidget()
        btn_container.setAttribute(Qt.WA_StyledBackground, True)
        btn_container.setSizePolicy(QSizePolicy.Maximum, QSizePolicy.Fixed)
        btn_container.setFixedHeight(36)  
        btn_container.setStyleSheet("""
            QWidget {
                background-color: #080808;
                border-radius: 8px;
                border: 1px solid #363636;
                padding: 0px 8px;
            }
        """)
        btn_layout = QHBoxLayout(btn_container)
        btn_layout.setContentsMargins(4, 0, 4, 0)
        btn_layout.setSpacing(8)
        btn_layout.setSizeConstraint(QHBoxLayout.SetFixedSize)

        new_btn = QPushButton("  New Strategy")
        new_btn.setIcon(QIcon("../../resources/images/plus.svg"))
        load_btn = QPushButton("  Load Strategy")
        load_btn.setIcon(QIcon("../../resources/images/import.svg"))

        for btn in (new_btn, load_btn):
            btn.setCursor(Qt.PointingHandCursor)
            btn.setFixedHeight(36)
            btn.setFont(QFont("Inter", 10, QFont.Medium))
            btn.setStyleSheet("""
                QPushButton {
                    background-color: transparent;
                    color: #FFFFFF;
                    border: none;
                }
            """)
            btn.setIconSize(QSize(16, 16))
            btn_layout.addWidget(btn)
        
        sep = QFrame()
        sep.setFrameShape(QFrame.VLine)
        sep.setFixedHeight(12)
        sep.setFixedWidth(2)
        sep.setStyleSheet("background-color: #363636;")
        btn_layout.insertWidget(1, sep, 0, Qt.AlignVCenter)

        layout.addWidget(btn_container, 0, Qt.AlignRight)

class ActionBar(QWidget):
    def __init__(self):
        super().__init__()
        self.setStyleSheet("background: #101010;")

        layout = QHBoxLayout(self)
        layout.setContentsMargins(16, 24, 16, 24)
        layout.setSpacing(10)

        spacer = QWidget()
        spacer.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)
        spacer.setStyleSheet("background-color: #080808")

        self.save_btn = QPushButton("Save Strategy")
        self.save_btn.setIcon(QIcon("../../resources/images/save.svg"))
        self.delete_btn = QPushButton("Delete Strategy")
        self.delete_btn.setIcon(QIcon("../../resources/images/delete.svg"))

        for btn in (self.save_btn, self.delete_btn):
            btn.setFont(QFont("Inter", 10, QFont.DemiBold))
            btn.setFixedHeight(40)
            btn.setCursor(Qt.PointingHandCursor)
            btn.setIconSize(QSize(20, 20))

        self.save_btn.setStyleSheet("""
                QPushButton {
                    background-color: #FFFFFF;
                    color: #080808;
                    border-radius: 8px;
                    padding: 0px 8px 0px 8px;
                }
        """)
        self.delete_btn.setStyleSheet("""
                QPushButton {
                    background-color: #261719;
                    color: #FF5D61;
                    border-radius: 8px;
                    padding: 0px 8px 0px 8px;
                }
        """)

        layout.addWidget(spacer)
        layout.addWidget(self.save_btn)
        layout.addWidget(self.delete_btn)

class FadeOverlay(QWidget):
    def __init__(self, parent):
        super().__init__(parent)
        self.setAttribute(Qt.WA_TransparentForMouseEvents)
        self.setAttribute(Qt.WA_NoSystemBackground)
        self.raise_()

    def paintEvent(self, event):
        painter = QPainter(self)
        gradient = QLinearGradient(0, 0, 0, self.height())
        gradient.setColorAt(0.0, QColor(0, 0, 0, 0))
        gradient.setColorAt(1.0, QColor("#080808"))
        painter.fillRect(self.rect(), gradient)

class CodeEditor(QsciScintilla):
    def __init__(self):
        super().__init__()
        font = QFont("Fira Code", 11, QFont.DemiBold)
        self.setUtf8(True)
        self.setFont(font)
        self.setPaper(QColor("#020101"))
        self.setBraceMatching(QsciScintilla.SloppyBraceMatch)
        self.setStyleSheet("border: none;")
        #self.setViewportMargins(220, 0, 220, 0)

        self.setIndentationGuides(True)
        self.setTabWidth(4)
        self.setIndentationsUseTabs(False)
        self.setAutoIndent(True)

        self.setAutoCompletionSource(QsciScintilla.AcsAll)
        self.setAutoCompletionThreshold(1) 
        self.setAutoCompletionCaseSensitivity(False)
        self.setAutoCompletionUseSingle(QsciScintilla.AcusNever)

        self.setCaretForegroundColor(QColor("white"))
        self.setCaretLineVisible(False)
        self.setCaretWidth(2)
        self.setCaretLineBackgroundColor(QColor("#2c313c"))
        
        self.setEolMode(QsciScintilla.EolWindows)
        self.setEolVisibility(False)

        self.setMarginType(0, QsciScintilla.NumberMargin)
        self.setMarginsFont(font)
        self.setMarginWidth(0, "000")
        self.setMarginWidth(2, "000")
        self.setMarginsForegroundColor(QColor("#999999"))
        self.setMarginsBackgroundColor(QColor("#080808"))

        lexer = QsciLexerPython(self)
        lexer.setDefaultPaper(QColor("#080808"))
        lexer.setDefaultFont(font)

        token_map = {
            QsciLexerPython.Default: "#FFFFFF",
            QsciLexerPython.Comment: "#227150",
            QsciLexerPython.CommentBlock: "#227150",
            QsciLexerPython.Number: "#2D61D9",
            QsciLexerPython.DoubleQuotedString: "#2D61D9",
            QsciLexerPython.SingleQuotedString: "#2D61D9",
            QsciLexerPython.TripleSingleQuotedString: "#2D61D9",
            QsciLexerPython.TripleDoubleQuotedString: "#2D61D9",
            QsciLexerPython.Keyword: "#7D2753",
            QsciLexerPython.ClassName: "#2D61D9",
            QsciLexerPython.FunctionMethodName: "#FFFFFF",
            QsciLexerPython.Operator: "#7D2753",
            QsciLexerPython.Identifier: "#FFFFFF",
            QsciLexerPython.UnclosedString: "#FF5D61",
            QsciLexerPython.HighlightedIdentifier:"#61afef",
            QsciLexerPython.Decorator: "#2D61D9"
        }
        for token, color in token_map.items():
            lexer.setColor(QColor(color), token)
            lexer.setPaper(QColor("#080808"), token)
            lexer.setFont(QFont("Fira Code", 11, QFont.DemiBold))
        
        self.setLexer(lexer)

        scroll_bar_style = """
            QScrollBar:vertical {
                background-color: #101010;
                width: 8px;
                margin: 0px;
                border: none;
            }
            QScrollBar::handle:vertical {
                background-color: #363636;
                min-height: 20px;
                border-radius: 4px;
            }
            QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
                height: 0px;
            }
            QScrollBar::add-page:vertical, QScrollBar::sub-page:vertical {
                background-color: #101010;
            }
        """
        self.verticalScrollBar().setStyleSheet(scroll_bar_style)
        self.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAsNeeded)
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        self.fade = FadeOverlay(self)
        self.max_chars = 130
        
    def resizeEvent(self, event):
        super().resizeEvent(event)
        fade_height = 160
        self.fade.setGeometry(
            0,
            self.height() - fade_height,
            self.width(),
            fade_height
        )

        if self.width() <= 0:
            return

        char_width = self.fontMetrics().horizontalAdvance("M")
        content_width = char_width * self.max_chars

        editor_width = self.width()

        margin = max(0, (editor_width - content_width) // 2)

        self.setViewportMargins(margin, 0, margin, 0)

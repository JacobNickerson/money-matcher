import os
import sqlite3
from pathlib import Path
from PyQt5.QtWidgets import ( 
    QWidget, QHBoxLayout, QPushButton, QLabel,
    QSizePolicy, QFrame, QComboBox, QFileDialog,
    QMessageBox, QDialog, QVBoxLayout, QLineEdit
)
from PyQt5.QtGui import (
    QFont, QColor, QPainter, QLinearGradient, QIcon
)
from PyQt5.QtCore import (
    Qt, QSize, pyqtSignal
)
from PyQt5.Qsci import (
    QsciScintilla, QsciLexerPython
)

class StrategyModal(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setModal(True)
        self.setMinimumWidth(600)
        self.setStyleSheet("""
            QDialog {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
            QLabel {
                color: #FFFFFF;
                background: transparent;
            }
            QPushButton {
                border-radius: 8px;
                padding: 10px 14px;
                font-size: 12px;
                font-weight: 600;
            }
        """)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        title = QLabel("New Strategy")
        title.setFont(QFont("Inter", 12, QFont.Medium))
        layout.addWidget(title)

        self.symbol_dropdown_widget, self.symbol_selected = self.symbol_list()
        self.name_input_widget, self.name_input = self.input_field("Strategy Name")
        layout.addWidget(self.symbol_dropdown_widget)
        layout.addWidget(self.name_input_widget)

        layout.addStretch()

        btn_layout = QHBoxLayout()
        btn_layout.setSpacing(12)

        cancel_btn = QPushButton("Cancel")
        cancel_btn.setMinimumHeight(32)
        cancel_btn.setMaximumHeight(44)
        cancel_btn.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        cancel_btn.setFont(QFont("Inter", 12, QFont.DemiBold))
        cancel_btn.setCursor(Qt.PointingHandCursor)
        cancel_btn.setStyleSheet("""
            QPushButton {
                background-color: #2B2B2B;
                color: #FFFFFF;
                border-radius: 8px;
                border: none;
            }
            QPushButton:hover {
                background-color: #3A3A3A;
            }
            QPushButton:pressed {
                background-color: #1F1F1F;
            }
        """)
        cancel_btn.clicked.connect(self.reject)

        submit_btn = QPushButton("Save")
        submit_btn.setMinimumHeight(32)
        submit_btn.setMaximumHeight(44)
        submit_btn.setSizePolicy(QSizePolicy.Policy.Expanding, QSizePolicy.Policy.Expanding)
        submit_btn.setFont(QFont("Inter", 12, QFont.DemiBold))
        submit_btn.setCursor(Qt.PointingHandCursor)
        submit_btn.setStyleSheet("""
            QPushButton {
                background-color: #FFFFFF;
                color: #080808;
                border-radius: 8px;
                border: none;
            }
            QPushButton:hover {
                background-color: #D9D9D9;
            }
            QPushButton:pressed {
                background-color: #D9D9D9;
            }
        """)
        submit_btn.clicked.connect(self.accept)

        btn_layout.addWidget(cancel_btn)
        btn_layout.addWidget(submit_btn)

        layout.addLayout(btn_layout)

    def input_field(self, label_text):
        container = QWidget()
        container.setAttribute(Qt.WA_StyledBackground, True)
        container.setStyleSheet("""
            QWidget {
                background-color: #101010;
            }
        """)
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        label = QLabel(label_text)
        label.setFont(QFont("Inter", 9))
        label.setStyleSheet("color: #999999;")

        field = QLineEdit()
        field.setFont(QFont("Inter", 10))
        field.setMinimumHeight(30)
        field.setMaximumHeight(36)

        field.setStyleSheet("""
            QLineEdit {
                background-color: #080808;
                border: 1px solid #363636;
                border-radius: 6px;
                padding-right: 10px;
                color: white;
            }
        """)
        field.setAlignment(Qt.AlignLeft | Qt.AlignVCenter)

        field_layout = QHBoxLayout()
        field_layout.setContentsMargins(0, 0, 0, 0)
        field_layout.addWidget(field)

        layout.addWidget(label)
        layout.addLayout(field_layout)

        return container, field
    
    def symbol_list(self):
        container = QWidget()
        container.setAttribute(Qt.WA_StyledBackground, True)
        container.setStyleSheet("""
            QWidget {
                background-color: #101010;
            }
        """)
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)
        
        label = QLabel("Symbol")
        label.setFont(QFont("Inter", 9))
        label.setStyleSheet("color: #999999;")

        list = QComboBox()
        list.setCursor(Qt.PointingHandCursor)
        list.setFont(QFont("Inter", 10))
        list.setMinimumHeight(30)
        list.setMaximumHeight(36)
        list.setStyleSheet("""
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
        """)
        list.addItems(["SOL/USD"])

        layout.addWidget(label)
        layout.addWidget(list)

        return container, list
    
    def get_data(self):
        name = self.name_input.text()
        symbol = self.symbol_selected.currentText()

        return name, symbol

class Header(QWidget):
    strategyChanged = pyqtSignal(str, str, str) 
    def __init__(self, editor):
        super().__init__()
        self.setAttribute(Qt.WA_StyledBackground, True)
        self.setStyleSheet("""
            Header {
                background-color: #101010;
                border: 1px solid #363636;
                border-radius: 16px;
            }
        """)
        self.initDatabase()
        self.editor = editor

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
        self.symbol_list.setCursor(Qt.PointingHandCursor)
        self.symbol_list.setMinimumWidth(200)
        layout.addWidget(self.symbol_list)

        self.strategy_list = QComboBox()
        self.strategy_list.setFont(QFont("Inter", 10, QFont.Medium))
        self.strategy_list.setStyleSheet(combo_style)
        self.strategy_list.setCursor(Qt.PointingHandCursor)
        self.strategy_list.setMinimumWidth(400)
        self.strategy_list.currentIndexChanged.connect(self.loadSelectedStrategy)
        layout.addWidget(self.strategy_list)

        self.symbol_list.currentIndexChanged.connect(self.refreshStrategyList)

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

        new_btn.clicked.connect(self.createNewStrategy)
        load_btn.clicked.connect(self.openFileDialog)
        
        sep = QFrame()
        sep.setFrameShape(QFrame.VLine)
        sep.setFixedHeight(12)
        sep.setFixedWidth(2)
        sep.setStyleSheet("background-color: #363636;")
        btn_layout.insertWidget(1, sep, 0, Qt.AlignVCenter)

        layout.addWidget(btn_container, 0, Qt.AlignRight)

    def getStrategiesFolder(self):
        root_dir = Path(__file__).resolve().parents[2]
        strategies_dir = root_dir / "strategies"
        strategies_dir.mkdir(parents=True, exist_ok=True)
        return strategies_dir
    
    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def openFileDialog(self):
        dialog = StrategyModal(self)

        if dialog.exec():
            strategies_dir = self.getStrategiesFolder()

            file_path, _ = QFileDialog.getOpenFileName(
                self,
                "Load Strategy",
                str(strategies_dir),
                "Python Files (*.py)"
            )

            if file_path:
                try:
                    with open(file_path, "r", encoding="utf-8") as f:
                        content = f.read()

                    self.editor.setText(content)
                    self.current_file = file_path

                except Exception as e:
                    print("Error loading file:", e)

    def createNewStrategy(self):
        dialog = StrategyModal(self)

        if dialog.exec():
            strategy_name, symbol = dialog.get_data()
            strategies_dir = self.getStrategiesFolder()

            default_name = strategy_name.lower().replace(" ", "_") + ".py"

            file_path, _ = QFileDialog.getSaveFileName(
                self,
                "Create New Strategy",
                str(strategies_dir / default_name),
                "Python Files (*.py)"
            )

            if not file_path:
                return

            content = f"""class {strategy_name.replace(" ", "")}:
    symbol = "{symbol}"

    def on_start(self):
        pass

    def on_book(self, book):
        pass

    def on_trade(self, trade):
        pass

    def on_fill(self, fill):
        pass

    def on_timer(self, now):
        pass

    def on_stop(self):
        pass
    """

            with open(file_path, "w", encoding="utf-8") as f:
                f.write(content)

            self.editor.setText(content)
            self.current_file = file_path

            self.saveStrategyToDatabase(strategy_name, symbol, file_path)
            self.refreshStrategyList()

    def loadSelectedStrategy(self):
        file_path = self.strategy_list.currentData()

        if not file_path:
            return

        try:
            with open(file_path, "r", encoding="utf-8") as f:
                content = f.read()

            self.editor.setText(content)
            name = self.strategy_list.currentText()
            symbol = self.symbol_list.currentText()
            self.strategyChanged.emit(name, symbol, file_path)
            self.current_file = file_path

        except Exception as e:
            print("Error loading strategy:", e)

    def initDatabase(self):
        db_path = self.getDatabasePath()

        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()

        cursor.execute("""
            CREATE TABLE IF NOT EXISTS strategies (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE
            )
        """)

        conn.commit()
        conn.close()

    def saveStrategyToDatabase(self, name, symbol, file_path):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            INSERT OR REPLACE INTO strategies (name, symbol, file_path)
            VALUES (?, ?, ?)
        """, (name, symbol, file_path))

        conn.commit()
        conn.close()

    def getSavedSymbols(self):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            SELECT DISTINCT symbol
            FROM strategies
            ORDER BY symbol
        """)

        rows = cursor.fetchall()
        conn.close()

        return [row[0] for row in rows]

    def getStrategiesBySymbol(self, symbol):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            SELECT id, name, symbol, file_path
            FROM strategies
            WHERE symbol = ?
            ORDER BY name
        """, (symbol,))

        rows = cursor.fetchall()
        conn.close()
        return rows
    
    def loadSymbols(self, selected_symbol=None):
        self.symbol_list.blockSignals(True)
        current_symbol = selected_symbol or self.symbol_list.currentText()
        self.symbol_list.clear()

        symbols = self.getSavedSymbols()
        if not symbols:
            symbols = ["SOL/USD"]
        self.symbol_list.addItems(symbols)

        if current_symbol in symbols:
            self.symbol_list.setCurrentText(current_symbol)

        self.symbol_list.blockSignals(False)

        self.refreshStrategyList()

    def refreshStrategyList(self):
        selected_symbol = self.symbol_list.currentText()

        self.strategy_list.blockSignals(True)
        self.strategy_list.clear()

        if not selected_symbol:
            self.strategy_list.blockSignals(False)
            return

        strategies = self.getStrategiesBySymbol(selected_symbol)

        for _, name, symbol, file_path in strategies:
            self.strategy_list.addItem(name, file_path)

        self.strategy_list.blockSignals(False)

        if self.strategy_list.count() > 0:
            self.strategy_list.setCurrentIndex(0)
            self.loadSelectedStrategy()

class ActionBar(QWidget):
    def __init__(self, editor, header):
        super().__init__()
        self.setStyleSheet("background: #101010;")
        self.editor = editor
        self.header = header

        self.current_file = None
        self.current_strategy_name = None
        self.current_symbol = None

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

        self.save_btn.clicked.connect(self.saveStrategy)
        self.delete_btn.clicked.connect(self.deleteStrategy)

        layout.addWidget(spacer)
        layout.addWidget(self.save_btn)
        layout.addWidget(self.delete_btn)

    def getStrategiesFolder(self):
        root_dir = Path(__file__).resolve().parents[2]
        strategies_dir = root_dir / "strategies"
        strategies_dir.mkdir(parents=True, exist_ok=True)
        return strategies_dir

    def getDatabasePath(self):
        root_dir = Path(__file__).resolve().parents[2]
        data_dir = root_dir / "data"
        data_dir.mkdir(parents=True, exist_ok=True)
        return data_dir / "matchmakers.db"

    def saveStrategyToDatabase(self, name, symbol, file_path):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            INSERT OR REPLACE INTO strategies (name, symbol, file_path)
            VALUES (?, ?, ?)
        """, (name, symbol, file_path))

        conn.commit()
        conn.close()

    def deleteStrategyFromDatabase(self, file_path):
        conn = sqlite3.connect(self.getDatabasePath())
        cursor = conn.cursor()

        cursor.execute("""
            DELETE FROM strategies
            WHERE file_path = ?
        """, (file_path,))

        conn.commit()
        conn.close()

    def setCurrentStrategy(self, name, symbol, file_path):
        self.current_strategy_name = name
        self.current_symbol = symbol
        self.current_file = file_path

    def saveStrategy(self):
        if not self.current_file:
            QMessageBox.warning(None, "No Strategy", "There is no strategy currently open.")
            return

        try:
            content = self.editor.text()

            with open(self.current_file, "w", encoding="utf-8") as f:
                f.write(content)

            strategy_name = self.current_strategy_name
            if not strategy_name:
                strategy_name = Path(self.current_file).stem

            symbol = self.current_symbol
            if not symbol:
                symbol = ""

            self.saveStrategyToDatabase(strategy_name, symbol, self.current_file)

            QMessageBox.information(None, "Saved", f"{strategy_name} was saved successfully.")
            print("Saved:", self.current_file)

        except Exception as e:
            QMessageBox.critical(None, "Save Error", f"Could not save strategy:\n{e}")
            print("Save error:", e)

    def deleteStrategy(self):
        if not self.current_file:
            QMessageBox.warning(None, "No Strategy", "There is no strategy currently open.")
            return

        file_path = self.current_file
        file_name = os.path.basename(file_path)
        deleted_symbol = self.current_symbol

        reply = QMessageBox.question(
            None,
            "Confirm Delete",
            f"Are you sure you want to delete:\n\n{file_name}?",
            QMessageBox.Discard | QMessageBox.Cancel,
            QMessageBox.Cancel
        )

        if reply != QMessageBox.Discard:
            return

        try:
            if os.path.exists(file_path):
                os.remove(file_path)

            self.deleteStrategyFromDatabase(file_path)

            self.editor.clear()
            self.current_file = None
            self.current_strategy_name = None
            self.current_symbol = None

            if self.header:
                self.header.loadSymbols(selected_symbol=deleted_symbol)

            QMessageBox.information(None, "Deleted", f"{file_name} was deleted.")
            print("Deleted:", file_path)

        except Exception as e:
            QMessageBox.critical(None, "Error", f"Could not delete file:\n{e}")


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

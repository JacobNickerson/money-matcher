from PyQt5.QtGui import (
    QFont, QColor
)
from PyQt5.QtCore import (
    Qt, QAbstractTableModel, QModelIndex
)

class TradeHistoryModel(QAbstractTableModel):
    headers = [
        "Symbol", "Date", "Type", "Side",
        "Price", "Amount", "Filled", "Total", "Status", "Action"
    ]

    def __init__(self, rows):
        super().__init__()
        self.rows = rows

    def rowCount(self, parent=QModelIndex()):
        return len(self.rows)

    def columnCount(self, parent=QModelIndex()):
        return len(self.headers)

    def data(self, index, role):
        if not index.isValid():
            return None

        row = self.rows[index.row()]
        col = self.headers[index.column()]

        if role == Qt.DisplayRole:
            return row.get(col, "")
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font

        if role == Qt.ForegroundRole and col == "Side":
            return QColor("#27AE60") if row["Side"] == "Buy" else QColor("#EB5757")

        if role == Qt.TextAlignmentRole:
            return Qt.AlignLeft | Qt.AlignVCenter

        return None

    def headerData(self, section, orientation, role):
        if orientation == Qt.Horizontal:
            if role == Qt.DisplayRole:
                return self.headers[section]

        if role == Qt.TextAlignmentRole:
            return Qt.AlignLeft | Qt.AlignVCenter
        
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font
        
        return None
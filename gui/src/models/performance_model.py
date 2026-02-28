from PyQt5.QtGui import (
    QFont, QColor
)
from PyQt5.QtCore import (
    Qt, QAbstractTableModel, QModelIndex
)

class PerformanceModel(QAbstractTableModel):
    headers = [
        "Date", "Strategy", "Symbol", "Type", "Profit/Loss", "% Gain/Loss", "Entry Price", "Exit Price"
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

        if role == Qt.ForegroundRole:
            if col == "Profit/Loss":
                return QColor("#EB5757") if row["Profit/Loss"][0] == "-" else QColor("#00C278")
            elif col == "% Gain/Loss":
                return QColor("#EB5757") if row["Profit/Loss"][0] == "-" else QColor("#00C278")

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
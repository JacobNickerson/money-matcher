from PyQt5.QtGui import (
    QFont, QColor
)
from PyQt5.QtCore import (
    Qt, QAbstractTableModel, QModelIndex
)

class OrderBookModel(QAbstractTableModel):
    headers = ["Price", "Amount", "Total"]

    def __init__(self, asks=None, bids=None, max_levels=7):
        super().__init__()
        self.max_levels = max_levels
        self.asks = (asks or [])[:max_levels]
        self.bids = (bids or [])[:max_levels]
        self.last_mid_price = None
        self.mid_direction = 0

    def rowCount(self, parent=QModelIndex()):
        return len(self.asks) + 1 + len(self.bids)
    
    def columnCount(self, parent=QModelIndex()):
        return len(self.headers)
    
    def headerData(self, section, orientation, role):
        if orientation == Qt.Horizontal:
            if role == Qt.DisplayRole:
                return self.headers[section]
            
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font
        
        return None
    
    def row_data(self, row):
        if row < len(self.asks):
            return "ask", self.asks[row]
        elif row == len(self.asks):
            return "mid", (self.mid_price(), self.spread(), 0)
        else:
            return "bid", self.bids[row - len(self.asks) - 1]
        
    def data(self, index, role):
        if not index.isValid():
            return None
        
        side, (price, amount, total) = self.row_data(index.row())
        column = index.column()

        if role == Qt.DisplayRole:
            if side == "mid":
                if column == 0 and price is not None:
                    return f"{price:.2f}"
                elif column == 1 and self.spread() is not None:
                    return f"{self.spread():.2f}"
                return ""
            if column == 0:
                return f"{price:.2f}"
            elif column == 1:
                return f"{amount:.4f}"
            elif column == 2:
                return f"{total:.2f}"
            
        if role == Qt.ForegroundRole and column == 0:
            if side == "ask":
                return QColor("#EB5757")
            elif side == "bid":
                return QColor("#27AE60")
            elif side == "mid":
                if self.mid_direction > 0:
                    return QColor("#27AE60")
                elif self.mid_direction < 0:
                    return QColor("#EB5757")
                else:
                    return QColor("#999999")
            
        if role == Qt.TextAlignmentRole:
            if column == 0:
                return Qt.AlignLeft | Qt.AlignVCenter
            elif column == 1:
                return Qt.AlignCenter | Qt.AlignVCenter
            elif column == 2:
                return Qt.AlignRight | Qt.AlignVCenter
            
        if role == Qt.FontRole:
            font = QFont("Inter", 10)
            font.setWeight(QFont.Medium)
            return font
            
    def row_info(self, row):
        side, (price, amount, total) = self.row_data(row)
        return side, price, amount, total
    
    def max_amount(self):
        amounts = [a[1] for a in self.asks + self.bids]
        return max(amounts) if amounts else 1
    
    def mid_price(self):
        if not self.asks or not self.bids:
            return None
        
        best_ask = self.asks[0][0]
        best_bid = self.bids[0][0]
        mid = (best_ask + best_bid) / 2

        if self.last_mid_price is not None:
            if mid > self.last_mid_price:
                self.mid_direction = 1
            elif mid < self.last_mid_price:
                self.mid_direction = -1
            else:
                self.mid_direction = 0

        self.last_mid_price = mid
        return mid
    
    def spread(self):
        if not self.asks or not self.bids:
            return None
        return self.asks[0][0] - self.bids[0][0]
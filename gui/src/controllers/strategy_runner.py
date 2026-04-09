import importlib.util
import time
from dataclasses import dataclass
from PyQt5.QtCore import (
    QObject, pyqtSignal
)

@dataclass
class Trade:
    symbol: str
    price: float
    qty: int
    side: str
    timestamp: int

@dataclass
class BookState:
    best_bid: float
    best_ask: float
    spread: float
    mid_price: float

class StrategyRunner(QObject):
    strategy_log = pyqtSignal(str)

    def __init__(self, strategy_file_path, bot_config, order_entry=None):
        super().__init__()

        self.strategy_file_path = strategy_file_path
        self.bot_config = bot_config
        self.order_entry = order_entry

        self.bot_id = bot_config["id"]
        self.bot_name = bot_config.get("bot_name", bot_config.get("name", f"bot_{self.bot_id}"))
        self.symbol = bot_config.get("symbol", "")

        self.order_size = float(bot_config.get("order_size", 1.0))
        self.max_position = float(bot_config.get("max_position", 100.0))
        self.latency_ms = int(bot_config.get("latency", bot_config.get("latency_ms", 0)))
        self.jitter_ms = int(bot_config.get("jitter", bot_config.get("jitter_ms", 0)))

        self.strategy = None
        self.status = "Paused"

        self.position = 0.0
        self.book_state = None
        self.last_trade = None
        self.last_fill = None
        self.started_at = None
        self.trades_processed = 0
        self.last_trade_time = None

    def load_strategy(self):
        try:
            spec = importlib.util.spec_from_file_location(
                f"strategy_{self.bot_id}",
                self.strategy_file_path
            )

            if spec is None or spec.loader is None:
                raise ValueError(f"Could not load strategy file: {self.strategy_file_path}")

            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            strategy_class = None
            for name in dir(module):
                obj = getattr(module, name)
                if (
                    isinstance(obj, type)
                    and hasattr(obj, "on_start")
                    and hasattr(obj, "on_book")
                    and hasattr(obj, "on_trade")
                    and hasattr(obj, "on_fill")
                    and hasattr(obj, "on_timer")
                    and hasattr(obj, "on_stop")
                ):
                    strategy_class = obj
                    break

            if strategy_class is None:
                raise ValueError(f"No valid strategy class found in {self.strategy_file_path}")

            self.strategy = strategy_class()
            self.strategy.bot = self
            self.strategy.bot_name = self.bot_name
            self.strategy.symbol = self.symbol

        except Exception as e:
            self.log(f"Error loading strategy from {self.strategy_file_path}: {e}")
            raise

    def start(self):
        if self.status == "Active":
            return

        if self.strategy is None:
            self.load_strategy()

        self.status = "Active"
        self.started_at = time.time()
        self.log(f"Started strategy for {self.bot_name}")
        self.strategy.on_start()

    def stop(self):
        if self.status != "Active":
            return

        self.status = "Paused"
        self.strategy.on_stop()
        self.log(f"Stopped strategy for {self.bot_name}")

    def on_market_event(self, event, rust_book):
        if self.status != "Active" or self.strategy is None:
            return

        try:
            old_book = self.book_state
            self.on_order_book_update(rust_book)

            if self.book_state is not None:
                if (self.book_state.best_bid is not None
                    and self.book_state.best_ask is not None
                    and self.book_state.spread is not None):
                    changed = (old_book is None
                        or old_book.best_bid != self.book_state.best_bid
                        or old_book.best_ask != self.book_state.best_ask
                        or old_book.spread != self.book_state.spread
                        or old_book.mid_price != self.book_state.mid_price)
                    if changed:
                        self.strategy.on_book(self.book_state)

                if type(event.kind).__name__ == "Trade":
                    try:
                        trade = Trade(
                            symbol=self.symbol,
                            price=float(event.kind.price),
                            qty=int(event.kind.quantity),
                            side="BUY" if "Bid" in str(event.kind.aggressor_side) else "SELL",
                            timestamp=int(event.timestamp),
                        )

                        self.trades_processed += 1
                        self.last_trade_time = time.time()
                        self.last_trade = trade

                        self.strategy.on_trade(trade)
                    except (AttributeError, ValueError) as e:
                        print(f"Error parsing trade event: {e}")

        except Exception as e:
            self.log(f"Error handling market event: {e}")

    def on_order_book_update(self, book):
        if self.status != "Active" or self.strategy is None:
            return

        try:
            best_bid = book.best_bid()
            best_ask = book.best_ask()
            spread = book.spread()
            mid_price = book.mid_price()

            self.book_state = BookState(
                best_bid=best_bid,
                best_ask=best_ask,
                spread=spread,
                mid_price=mid_price
            )

        except Exception as e:
            self.book_state = None
            self.log(f"Error processing order book update: {e}")

    def submit_order(self, side, price, qty):
        if self.status != "Active":
            self.log("Order ignored: strategy is paused")
            return

        try:
            qty = float(qty)

            if side.upper() == "BUY" and self.position + qty > self.max_position:
                self.log("BUY blocked: max position exceeded")
                return

            if side.upper() == "SELL" and self.position - qty < -self.max_position:
                self.log("SELL blocked: max position exceeded")
                return

            if self.order_entry is not None:
                self.order_entry.submit_order(
                    bot_order=True,
                    _side=side,
                    _price=price,
                    _qty=qty
                )
                self.log(f"Submitted {side} order: qty={qty}, price={price}")

        except Exception as e:
            self.log(f"Order submit error: {e}")

    def buy(self, qty=None, price=None):
        qty = self.order_size if qty is None else qty
        if price is None and self.book_state is not None:
            price = self.book_state.best_ask
        self.submit_order("BUY", price, qty)

    def sell(self, qty=None, price=None):
        qty = self.order_size if qty is None else qty
        if price is None and self.book_state is not None:
            price = self.book_state.best_bid
        self.submit_order("SELL", price, qty)

    def on_fill(self, fill):
        if self.status != "Active" or self.strategy is None or fill is None:
            return

        self.last_fill = fill

        try:
            side = getattr(fill, "side", None)
            qty = float(getattr(fill, "qty", 0))

            if side == "BUY":
                self.position += qty
            elif side == "SELL":
                self.position -= qty

            self.strategy.on_fill(fill)
            self.log(f"Fill received: {side} {qty}, new position={self.position}")

        except Exception as e:
            self.log(f"Error handling fill: {e}")

    def on_trade(self, trade):
        if self.status != "Active" or self.strategy is None or trade is None:
            return

        self.last_trade = trade

        try:
            self.strategy.on_trade(trade)
        except Exception as e:
            self.log(f"Error handling trade: {e}")

    def on_timer(self, now=None):
        if self.status != "Active" or self.strategy is None:
            return

        if now is None:
            now = time.time()

        try:
            self.strategy.on_timer(now)
        except Exception as e:
            self.log(f"Error handling timer: {e}")

    def get_stats(self):
        return {
            "bot_id": self.bot_id,
            "bot_name": self.bot_name,
            "symbol": self.symbol,
            "strategy_name": self.bot_config.get("strategy_name", "Unknown strategy"),
            "status": self.status,
            "position": self.position,
            "order_size": self.order_size,
            "max_position": self.max_position,
            "latency_ms": self.latency_ms,
            "jitter_ms": self.jitter_ms,
            "last_trade_price": getattr(self.last_trade, "price", None) if self.last_trade else None,
            "best_bid": self.book_state.best_bid if self.book_state else None,
            "best_ask": self.book_state.best_ask if self.book_state else None,
        }

    def log(self, message):
        self.strategy_log.emit(f"[{self.bot_name}] {message}")
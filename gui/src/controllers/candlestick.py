from dataclasses import dataclass
import pandas as pd

@dataclass
class Candle:
    time: pd.Timestamp
    open: float
    high: float
    low: float
    close: float
    volume: float

class CandlestickAggregator:
    def __init__(self, interval_seconds=60):
        self.interval_seconds = interval_seconds
        self.current_candle = None
        self.history = []

    def bucket_start(self, ts_ns):
        ts = pd.to_datetime(ts_ns, unit="ns", utc=True)
        return ts.floor(f"{self.interval_seconds}s")

    def process_trade(self, timestamp_ns, price, quantity):
        bucket_time = self.bucket_start(timestamp_ns)

        if self.current_candle is None:
            self.current_candle = Candle(
                bucket_time, price, price, price, price, quantity
            )
            return None

        if self.current_candle.time == bucket_time:
            self.current_candle.high = max(self.current_candle.high, price)
            self.current_candle.low = min(self.current_candle.low, price)
            self.current_candle.close = price
            self.current_candle.volume += quantity
            return None

        finished = {
            "time": self.current_candle.time,
            "open": self.current_candle.open,
            "high": self.current_candle.high,
            "low": self.current_candle.low,
            "close": self.current_candle.close,
            "volume": self.current_candle.volume,
        }
        self.history.append(finished)

        self.current_candle = Candle(
            bucket_time, price, price, price, price, quantity
        )
        return finished

    def current_as_dict(self):
        if self.current_candle is None:
            return None
        return {
            "time": self.current_candle.time,
            "open": self.current_candle.open,
            "high": self.current_candle.high,
            "low": self.current_candle.low,
            "close": self.current_candle.close,
            "volume": self.current_candle.volume,
        }

    def dataframe(self):
        rows = list(self.history)
        if self.current_candle is not None:
            rows.append(self.current_as_dict())
        return pd.DataFrame(rows)


class CandlestickController:
    def __init__(self, chart):
        self.chart = chart
        self.symbol = "SOL/USD"
        self.aggregator = CandlestickAggregator(interval_seconds=5)
        self.full_refresh = False
        self.partial = None

        self.chart.layout(background_color="#101010", text_color="#999999", font_family="Inter")
        self.chart.candle_style(
            up_color="#00C278",
            down_color="#EB5757",
            wick_up_color="#00C278",
            wick_down_color="#EB5757",
            border_visible=False,
        )
        self.chart.grid(color="#080808")
        self.chart.legend(
            visible=True,
            font_size=14,
            font_family="Inter",
            color="#999999",
            color_based_on_candle=True,
            text=self.symbol,
            lines=False,
        )
        self.chart.price_scale(border_visible=False)
        self.chart.time_scale(border_visible=False)

        self.chart.set(pd.DataFrame(columns=["time", "open", "high", "low", "close", "volume"]))

    def handle_market_event(self, event):
        if not event.kind.is_trade():
            return

        price = float(event.kind.trade_price())
        quantity = float(event.kind.trade_quantity())

        finished = self.aggregator.process_trade(
            timestamp_ns=event.timestamp,
            price=price,
            quantity=quantity,
        )

        if finished is not None:
            self.full_refresh = True
        else:
            self.partial = self.aggregator.current_as_dict()

    def refresh_chart(self):
        if self.full_refresh:
            df = self.aggregator.dataframe()
            self.chart.set(df)
            self.full_refresh = False
            self.partial = None
        elif self.partial is not None:
            df = self.aggregator.dataframe()
            if len(df) == 1:
                self.chart.set(df)
            else:
                row = pd.Series(self.partial)
                self.chart.update(row)
            self.partial = None
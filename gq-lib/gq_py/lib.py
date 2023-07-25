import ctypes
import json
import time
from enum import Enum
from sys import platform

class Exchanges(Enum): 
    Coinbase = "coinbase"
    Kraken   = "kraken"
    Bitfinex = "bitfinex"
    Binance  = "binance"
    
class AssetClass(Enum):
    Spot = "spot"    
    

class DataType(Enum):
    Ticker  = "ticker"
    Book    = "book"
    Trade   = "trade"
    

class GqLive:
    __lib: ctypes.CDLL
    __receiver_ptr: int
    # Initialize the object, 
    def __init__(self) -> any:                
        if platform == "darwin":
            self.__lib = ctypes.cdll.LoadLibrary('../target/debug/libgq_rust.dylib')
            # Define the C function signatures
            self.__lib.create_socket.restype = ctypes.c_void_p
            self.__lib.create_socket.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
            self.__lib.receive_message.argtypes = [ctypes.c_void_p]
            self.__lib.receive_message.restype = ctypes.c_char_p
            self.__lib.destroy_socket.argtypes = [ctypes.c_void_p]            
            self.__lib.create_csv.argtypes = [ctypes.c_void_p]
        elif platform == "linux":
            pass
        elif platform == "windows":
            pass
        
    def connect(self, exchange: Exchanges, asset_class: AssetClass, data_type: DataType, symbol: str) -> any:
        b_exchange = exchange.value.encode("utf-8")
        b_asset_class = asset_class.value.encode("utf-8")
        b_data_type = data_type.value.encode("utf-8")
        b_symbol = symbol.encode("utf-8")
        
        # Call the Rust create_socket function to create a socket and get a pointer to the receiver channel    
        
        self.__receiver_ptr = self.__lib.create_socket(b_exchange, b_asset_class, b_data_type, b_symbol)            
        return self
        
    def read(self) -> str: 
        message = self.__lib.receive_message(self.__receiver_ptr)
        message_str: str = message.decode("utf-8")     
        return message_str
    
    def disconnect(self):
        self.__lib.destroy_socket(self.__receiver_ptr)
        
    # will convert into json and write as a CSV
    def write_csv(self, list):
        j_list: str = json.dumps(list)
        b_list = j_list.encode("utf-8")
        # c_json_list = ctypes
        self.__lib.create_csv(b_list)
        
        
coin = GqLive().connect(Exchanges.Coinbase, AssetClass.Spot, DataType.Ticker, "BTC-USD")

coin_list = []

t_end = time.time() + 1
while time.time() < t_end:
    kra_res = coin.read()
    coin_list.append(json.loads(kra_res))

coin.write_csv(coin_list)

# print(coin_list)
# print(len(coin_list))
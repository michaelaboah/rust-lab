import ctypes
from lib import GqLive



# Load the Rust shared library
lib = lib = ctypes.cdll.LoadLibrary('target/debug/libgq_rust.dylib')

# Define the function signature
test_convert = lib.test_convert
                        # exchange        asset_class      data_typee       symbol
test_convert.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]
test_convert.restype = None

# Call the function with a Python string
arg1 = "Arg1!"
b_arg1 = arg1.encode("utf-8")

arg2 = "Arg2!"
b_arg2 = arg2.encode("utf-8")

arg3 = "Arg3!"
b_arg3 = arg3.encode("utf-8")

arg4 = "Arg4!"
b_arg4 = arg4.encode("utf-8")

test_convert(b_arg1, b_arg2, b_arg3, b_arg4)

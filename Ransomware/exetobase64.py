import base64

# Path to the destructive.exe file
file_path = 'destructive.exe'

# Read the file in binary mode
with open(file_path, 'rb') as file:
    file_data = file.read()

# Encode the binary data to base64
encoded_data = base64.b64encode(file_data)

# Convert the base64 bytes to a string
encoded_str = encoded_data.decode('utf-8')

# Print or save the base64 encoded string
print(encoded_str)

# Optionally, save to a file (e.g., destructive_base64.txt)
with open('destructive_base64.txt', 'w') as base64_file:
    base64_file.write(encoded_str)

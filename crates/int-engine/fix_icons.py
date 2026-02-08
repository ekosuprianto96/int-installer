import base64
import os

# Minimal 1x1 red dot PNG
data = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
icon_bytes = base64.b64decode(data)

os.makedirs("icons", exist_ok=True)
for name in ["32x32.png", "128x128.png", "128x128@2x.png", "icon.png"]:
    with open(os.path.join("icons", name), "wb") as f:
        f.write(icon_bytes)

print("Minimalist icons generated")

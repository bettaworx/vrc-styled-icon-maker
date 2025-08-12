import os
from PIL import Image, ImageDraw, ImageFilter
import cv2
import numpy as np
import argparse

# ===== 設定部分 =====

start_color = (255, 255, 255)  # グラデーション開始色 (R,G,B)
end_color   = (192,192,192)  # グラデーション終了色 (R,G,B)

shadow_offset = (-1,1)             # ドロップシャドウの位置 (x, y)
shadow_color = (0, 0, 0, 255)      # 影の色 (R,G,B,A)
shadow_blur_radius = 3             # 影のぼかし半径
black_threshold = 50               # 黒検出の閾値（RGBがこの値以下なら黒とみなす）
final_canvas_size = (256, 256)     # 最終キャンバスサイズ
resize_height = 256                # 縦基準のリサイズ高さ
# ===================

def parse_args():
    parser = argparse.ArgumentParser(description='Image processing tool')
    parser.add_argument('-i', '--input', default="input_images",
                        help='Input folder path (default: input_images)')
    parser.add_argument('-o', '--output', default="output_images",
                        help='Output folder path (default: output_images)')
    return parser.parse_args()

def apply_gradient_to_black(img, start_color, end_color, gradient_start=0.0, gradient_end=1.0):
    width, height = img.size
    original_alpha = img.getchannel('A')

    gradient = Image.new("RGBA", (width, height))
    grad_pixels = gradient.load()   
    for y in range(height):
        ratio = (y / (height - 1))
        ratio = (ratio - gradient_start) / (gradient_end - gradient_start)
        ratio = max(0.0, min(1.0, ratio))

        r = int(start_color[0] * (1 - ratio) + end_color[0] * ratio)
        g = int(start_color[1] * (1 - ratio) + end_color[1] * ratio)
        b = int(start_color[2] * (1 - ratio) + end_color[2] * ratio)
        for x in range(width):
            grad_pixels[x, y] = (r, g, b, 255)

    result = gradient.copy()
    result.putalpha(original_alpha)
    
    return result

def add_drop_shadow_fit(image, offset=(4, 4), shadow_color=(0, 0, 0, 150), blur_radius=6):
    width, height = image.size

    extra_w = abs(offset[0]) + blur_radius * 2
    extra_h = abs(offset[1]) + blur_radius * 2
    canvas_size = (width + extra_w, height + extra_h)

    shadow_layer = Image.new("RGBA", canvas_size, (0, 0, 0, 0))

    cv_img = np.array(Image.composite(Image.new("RGB", image.size, (0, 0, 0)), image, mask=image))

    inverted = cv2.bitwise_not(cv_img)
    gray = cv2.cvtColor(inverted, cv2.COLOR_BGR2GRAY)
    _, binary = cv2.threshold(gray, 127, 255, cv2.THRESH_BINARY)
    kernel = np.ones((0,0), np.uint8)
    eroded = cv2.erode(binary, kernel, iterations=1)

    shadow_draw = Image.fromarray(eroded).convert("L")
    shadow_draw = shadow_draw.convert("RGBA")
    shadow_draw.putalpha(shadow_draw.getchannel('R'))
    shadow_alpha = shadow_draw.split()[3]
    shadow_pos = (offset[0] + blur_radius, offset[1] + blur_radius)
    shadow_layer.paste(shadow_draw, shadow_pos, mask=shadow_alpha)
    shadow_layer = shadow_layer.filter(ImageFilter.GaussianBlur(blur_radius))

    main_layer = Image.new("RGBA", canvas_size, (255,255,255, 0))
    main_layer.paste(image, (blur_radius, blur_radius))

    result = Image.alpha_composite(shadow_layer, main_layer)

    return result

def resize_height_keep_ratio(img, target_height):
    width, height = img.size
    if height == target_height:
        return img
    new_width = int(width * (target_height / height))
    return img.resize((new_width, target_height), Image.LANCZOS)

def expand_canvas_center(image, canvas_size=(512, 512)):
    canvas = Image.new("RGBA", canvas_size, (255,255,255, 0))
    img_w, img_h = image.size
    canvas_w, canvas_h = canvas_size

    x = (canvas_w - img_w) // 2
    y = (canvas_h - img_h) // 2

    canvas.paste(image, (x, y))
    return canvas

def main():
    args = parse_args()
    input_folder = args.input
    output_folder = args.output
    
    os.makedirs(output_folder, exist_ok=True)

    for filename in os.listdir(input_folder):
        if not filename.lower().endswith((".png", ".jpg", ".jpeg")):
            continue

        img = Image.open(os.path.join(input_folder, filename)).convert("RGBA")

        colored_icon = apply_gradient_to_black(img, start_color, end_color,0.2,0.8)

        shadowed = add_drop_shadow_fit(colored_icon, shadow_offset, shadow_color, shadow_blur_radius)

        expanded = expand_canvas_center(shadowed, final_canvas_size)

        expanded.save(os.path.join(output_folder, filename))

        print(f"[\tOK\t]\tSaved to {os.path.join(output_folder, filename)}")

if __name__ == "__main__":
    main()
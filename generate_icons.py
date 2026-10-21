import os
import math
from PIL import Image, ImageDraw

def create_base_icon(size=1024):
    """Draw high-res supersampled (1024x1024) ColLink branding icon."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # 1. Background gradient rounded rectangle
    # Draw gradient into a mask
    bg = Image.new("RGBA", (size, size))
    # Linear gradient #2563EB (37, 99, 235) to #3B82F6 (59, 130, 246)
    c1 = (37, 99, 235, 255)
    c2 = (59, 130, 246, 255)
    for y in range(size):
        for x in range(size):
            t = (x + y) / (2.0 * size)
            r = int(c1[0] * (1 - t) + c2[0] * t)
            g = int(c1[1] * (1 - t) + c2[1] * t)
            b = int(c1[2] * (1 - t) + c2[2] * t)
            bg.putpixel((x, y), (r, g, b, 255))

    mask = Image.new("L", (size, size), 0)
    mask_draw = ImageDraw.Draw(mask)
    corner_radius = int(size * 0.22) # ~225px
    mask_draw.rounded_rectangle([0, 0, size, size], radius=corner_radius, fill=255)

    img.paste(bg, (0, 0), mask)
    draw = ImageDraw.Draw(img)

    # Scale factor from 32x32 base reference
    scale = size / 32.0

    # 2. Source Table Node Card (Left)
    # rect x=4.5, y=6.5, width=9, height=12, rx=2
    x1, y1 = int(4.5 * scale), int(6.5 * scale)
    w1, h1 = int(9.0 * scale), int(12.0 * scale)
    rx1 = int(2.0 * scale)

    # White body
    draw.rounded_rectangle([x1, y1, x1 + w1, y1 + h1], radius=rx1, fill=(255, 255, 255, 245))
    # Header banner (DBEAFE)
    draw.rounded_rectangle([x1, y1, x1 + w1, y1 + int(3.5 * scale)], radius=rx1, fill=(219, 234, 254, 255))
    # Header bottom square-off
    draw.rectangle([x1, y1 + int(2.0 * scale), x1 + w1, y1 + int(3.5 * scale)], fill=(219, 234, 254, 255))
    # Divider line
    draw.line([x1, y1 + int(3.5 * scale), x1 + w1, y1 + int(3.5 * scale)], fill=(147, 197, 253, 255), width=max(1, int(0.8 * scale)))
    # Row lines
    row_w = max(2, int(1.2 * scale))
    draw.line([x1 + int(1.5 * scale), y1 + int(6.0 * scale), x1 + int(5.5 * scale), y1 + int(6.0 * scale)], fill=(148, 163, 184, 255), width=row_w)
    draw.line([x1 + int(1.5 * scale), y1 + int(8.8 * scale), x1 + int(6.8 * scale), y1 + int(8.8 * scale)], fill=(148, 163, 184, 255), width=row_w)

    # 3. Target Table Node Card (Right)
    # rect x=18.5, y=13.5, width=9, height=12, rx=2
    x2, y2 = int(18.5 * scale), int(13.5 * scale)
    w2, h2 = int(9.0 * scale), int(12.0 * scale)
    rx2 = int(2.0 * scale)

    # White body
    draw.rounded_rectangle([x2, y2, x2 + w2, y2 + h2], radius=rx2, fill=(255, 255, 255, 245))
    # Header banner (DBEAFE)
    draw.rounded_rectangle([x2, y2, x2 + w2, y2 + int(3.5 * scale)], radius=rx2, fill=(219, 234, 254, 255))
    draw.rectangle([x2, y2 + int(2.0 * scale), x2 + w2, y2 + int(3.5 * scale)], fill=(219, 234, 254, 255))
    # Divider line
    draw.line([x2, y2 + int(3.5 * scale), x2 + w2, y2 + int(3.5 * scale)], fill=(147, 197, 253, 255), width=max(1, int(0.8 * scale)))
    # Row lines
    draw.line([x2 + int(1.5 * scale), y2 + int(6.0 * scale), x2 + int(6.5 * scale), y2 + int(6.0 * scale)], fill=(148, 163, 184, 255), width=row_w)
    draw.line([x2 + int(1.5 * scale), y2 + int(8.8 * scale), x2 + int(5.5 * scale), y2 + int(8.8 * scale)], fill=(148, 163, 184, 255), width=row_w)

    # 4. Connecting Smooth Bezier Curve
    # M 13.5 15 C 16 15, 16 22, 18.5 22
    p0 = (13.5 * scale, 15.0 * scale)
    p1 = (16.0 * scale, 15.0 * scale)
    p2 = (16.0 * scale, 22.0 * scale)
    p3 = (18.5 * scale, 22.0 * scale)

    curve_pts = []
    steps = 60
    for i in range(steps + 1):
        t = i / float(steps)
        # Cubic Bezier: (1-t)^3 * P0 + 3*(1-t)^2*t * P1 + 3*(1-t)*t^2 * P2 + t^3 * P3
        cx = (1 - t)**3 * p0[0] + 3 * (1 - t)**2 * t * p1[0] + 3 * (1 - t) * t**2 * p2[0] + t**3 * p3[0]
        cy = (1 - t)**3 * p0[1] + 3 * (1 - t)**2 * t * p1[1] + 3 * (1 - t) * t**2 * p2[1] + t**3 * p3[1]
        curve_pts.append((cx, cy))

    line_w = int(2.2 * scale)
    for i in range(len(curve_pts) - 1):
        draw.line([curve_pts[i], curve_pts[i + 1]], fill=(255, 255, 255, 255), width=line_w)

    # 5. Anchor Ports
    def draw_port(cx, cy, r_outer, r_inner):
        draw.ellipse([cx - r_outer, cy - r_outer, cx + r_outer, cy + r_outer], fill=(37, 99, 235, 255))
        draw.ellipse([cx - r_inner, cy - r_inner, cx + r_inner, cy + r_inner], fill=(248, 250, 252, 255))

    r_out = int(1.8 * scale)
    r_in = int(1.1 * scale)
    draw_port(p0[0], p0[1], r_out, r_in)
    draw_port(p3[0], p3[1], r_out, r_in)

    # Midpoint pulse dot
    mid_cx, mid_cy = 16.0 * scale, 18.5 * scale
    r_mid = int(1.4 * scale)
    draw.ellipse([mid_cx - r_mid, mid_cy - r_mid, mid_cx + r_mid, mid_cy + r_mid], fill=(96, 165, 250, 255))

    return img

def main():
    print("Generating master 1024x1024 icon...")
    master = create_base_icon(1024)

    # Paths
    root_dir = os.path.dirname(os.path.abspath(__file__))
    frontend_dir = os.path.join(root_dir, "frontend")
    public_dir = os.path.join(frontend_dir, "public")
    dist_dir = os.path.join(frontend_dir, "dist")

    os.makedirs(public_dir, exist_ok=True)

    # 1. logo.png (256x256)
    img_256 = master.resize((256, 256), Image.Resampling.LANCZOS)
    img_192 = master.resize((192, 192), Image.Resampling.LANCZOS)
    
    logo_path = os.path.join(public_dir, "logo.png")
    img_256.save(logo_path, "PNG")
    print(f"Saved: {logo_path}")

    root_png_path = os.path.join(root_dir, "app_icon.png")
    img_256.save(root_png_path, "PNG")
    print(f"Saved: {root_png_path}")

    # 2. favicon.ico & app_icon.ico (with multi-sizes: 16, 32, 48, 64, 128, 256)
    ico_sizes = [(16, 16), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]

    favicon_ico_path = os.path.join(public_dir, "favicon.ico")
    master.save(favicon_ico_path, format="ICO", sizes=ico_sizes)
    print(f"Saved: {favicon_ico_path}")

    root_ico_path = os.path.join(root_dir, "app_icon.ico")
    master.save(root_ico_path, format="ICO", sizes=ico_sizes)
    print(f"Saved: {root_ico_path}")

    # 3. Clean up or replace vite.svg
    vite_svg_public = os.path.join(public_dir, "vite.svg")
    favicon_svg = os.path.join(public_dir, "favicon.svg")
    
    if os.path.exists(vite_svg_public):
        try:
            os.remove(vite_svg_public)
            print(f"Removed old: {vite_svg_public}")
        except Exception as e:
            print(f"Could not remove vite.svg: {e}")

    # If dist exists, sync assets
    if os.path.exists(dist_dir):
        import shutil
        shutil.copy2(favicon_ico_path, os.path.join(dist_dir, "favicon.ico"))
        shutil.copy2(logo_path, os.path.join(dist_dir, "logo.png"))
        if os.path.exists(favicon_svg):
            shutil.copy2(favicon_svg, os.path.join(dist_dir, "favicon.svg"))
        dist_vite = os.path.join(dist_dir, "vite.svg")
        if os.path.exists(dist_vite):
            os.remove(dist_vite)
        print("Synchronized new icons to frontend/dist/")

    print("All icons successfully generated!")

if __name__ == "__main__":
    main()

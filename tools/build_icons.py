"""Build transparent monochrome assets with the existing Pillow dependency.

Geometry is adapted from the public Zed application mark, with the inner
detail and all gradients/backgrounds removed. SVG is the editable source.
This is a build tool, never part of the resident runtime.
"""
from pathlib import Path
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'assets/icons'
OUT.mkdir(parents=True, exist_ok=True)
PATHS = (
    ((14, 65), (14, 14), (82, 14), (14, 82), (82, 82), (82, 44)),
    ((28, 55), (28, 28), (68, 28)),
    ((28, 68), (68, 68), (68, 41)),
)

def svg(color):
    lines = ''.join('<polyline points="' + ' '.join(f'{x},{y}' for x,y in path) + '"/>' for path in PATHS)
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 96 96" fill="none"><g stroke="{color}" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">{lines}</g></svg>'

def render(size, ink, outline=None, state=None):
    scale = 4
    canvas = Image.new('RGBA', (size*scale, size*scale), (0,0,0,0))
    draw = ImageDraw.Draw(canvas)
    extent = 80 if state else 96
    ratio = size*scale*extent/(96*96)
    offset = size*scale*0.035 if state else 0
    for path in PATHS:
        points = [(round(x*ratio+offset),round(y*ratio+offset)) for x,y in path]
        for color,width in ((outline,9),(ink,6)) if outline else ((ink,6),):
            stroke = max(1,round(width*ratio))
            draw.line(points, fill=color, width=stroke, joint='curve')
            radius=stroke/2
            for x,y in points:
                draw.ellipse((x-radius,y-radius,x+radius,y+radius),fill=color)
    if state:
        # Shape conveys status; color remains monochrome. Tooltips provide text.
        unit=size*scale/32
        def box(x1,y1,x2,y2): return tuple(v*unit for v in (x1,y1,x2,y2))
        inverse=(255,255,255,255) if ink[0]==0 else (0,0,0,255)
        draw.ellipse(box(23,23,31,31),fill=inverse)
        width=max(1,round(unit))
        if state=='active': draw.ellipse(box(25,25,29,29),fill=ink)
        elif state=='paused':
            draw.line([(25*unit,25*unit),(25*unit,29*unit)],fill=ink,width=width)
            draw.line([(28*unit,25*unit),(28*unit,29*unit)],fill=ink,width=width)
        elif state=='error':
            draw.line([(25*unit,25*unit),(29*unit,29*unit)],fill=ink,width=width)
            draw.line([(29*unit,25*unit),(25*unit,29*unit)],fill=ink,width=width)
        elif state=='unsupported':
            draw.line([(27*unit,24*unit),(27*unit,27*unit)],fill=ink,width=width)
            draw.ellipse(box(26.5,28,27.5,29),fill=ink)
        else: draw.ellipse(box(25,25,29,29),outline=ink,width=width)
    return canvas.resize((size,size),Image.Resampling.LANCZOS)

for name, ink in [('light',(0,0,0,255)),('dark',(255,255,255,255))]:
    (OUT/f'mark-{name}.svg').write_text(svg('black' if name=='light' else 'white'),encoding='utf-8')
    render(512,ink).save(OUT/f'mark-{name}.png')
    for state in ('waiting','active','paused','unsupported','error'):
        icon=render(32,ink,state=state)
        icon.save(OUT/f'tray-{name}-{state}.png')
        (OUT/f'tray-{name}-{state}.rgba').write_bytes(icon.tobytes())

render(256,(0,0,0,255),(255,255,255,255)).save(OUT/'app.ico',sizes=[(16,16),(20,20),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])
render(256,(0,0,0,255),(255,255,255,255)).save(OUT/'app.png')
sheet=Image.new('RGB',(960,440),'white')
for index,(background,name) in enumerate([('#fafafa','light'),('#161616','dark')]):
    panel=Image.new('RGB',(480,440),background)
    panel.paste(Image.open(OUT/f'mark-{name}.png').resize((220,220)),(130,36),Image.open(OUT/f'mark-{name}.png').resize((220,220)))
    for n,state in enumerate(('waiting','active','paused','unsupported','error')):
        icon=Image.open(OUT/f'tray-{name}-{state}.png').resize((48,48))
        panel.paste(icon,(55+n*82,330),icon)
    sheet.paste(panel,(index*480,0))
sheet.save(OUT/'preview.png')
print(f'Built transparent SVG, PNG, ICO and tray assets in {OUT}')

__author__ = 'shi'


import glob, os

import sys
from subprocess import call

import cStringIO
import PIL.Image

image_data = None

def imagetopy(image, output_file):
    with open(image, 'rb') as fin:
        image_data = fin.read()

    with open(output_file, 'w') as fout:
        fout.write('image_data = '+ repr(image_data))

def imageto64py(image, output_file):
    import base64
    with open(image, "rb") as image_file:
        encoded_string = base64.b64encode(image_file.read())
    with open(output_file, 'w') as fout:
        fout.write('image_data = '+ repr(encoded_string))

def pytoimage(pyfile):
    pymodule = __import__(pyfile)
    img = PIL.Image.open(cStringIO.StringIO(pymodule.image_data))
    img.show()

# if __name__ == '__main__':
#     imagetopy('spot.png', 'wishes.py')
    # pytoimage('wishes')

if __name__ == '__main__':
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    if len(sys.argv) == 1:
        call(["python", __file__, "OK"])
    elif sys.argv[1] == "OK":
        fimgs = glob.glob("*.gif")
        print fimgs
        for fimg in fimgs:
            fpy = fimg + ".py"
            if not os.path.exists(fpy):
                imageto64py(fimg, fpy)

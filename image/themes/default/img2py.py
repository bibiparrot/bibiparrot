__author__ = 'shi'


import glob, os
import sys
from PIL import Image
from subprocess import call

###
##  Very pool quality resize.
#
#
def resize(inf, x, y, ouf=None):
    # from PIL import Image
    MAXSIZEX = x # this is the maximum width of the images
    MAXSIZEY = y # this is the maximum height of the images
    ratio = 1. * MAXSIZEX / MAXSIZEY

    im = Image.open(inf) # open the input file
    (width, height) = im.size        # get the size of the input image
    if width == x and height == y:
        return inf
    ext = os.path.splitext(inf)[-1]
    if width > height * ratio:
        # crop the image on the left and right side
        newwidth = int(height * ratio)
        left = width / 2 - newwidth / 2
        right = left + newwidth
        # keep the height of the image
        top = 0
        bottom = height
    elif width < height * ratio:
        # crop the image on the top and bottom
        newheight = int(width * ratio)
        top = height / 2 - newheight / 2
        bottom = top + newheight
        # keep the width of the impage
        left = 0
        right = width
    if width != height * ratio:
        im = im.crop((left, top, right, bottom))
    if ouf is None:
        ouf = os.path.join(os.path.dirname(inf), 'resized_'+os.path.basename(inf))
    fout = open(ouf,'wb')
    im = im.resize((MAXSIZEX, MAXSIZEY), Image.ANTIALIAS)
    im.save(fout, ext.replace('.',''), quality = 100) # save the image
    fout.close()
    return ouf



def write_index_category(pyf, imgf, imgName):
    pyfp = open(pyf,'a')
    nam,ext = os.path.splitext(os.path.basename(imgf))
    try:
        pyfp.write("\n\nindex.append('"+nam+"')\ncatalog['"+nam+"'] = "+imgName+"\n")
    finally:
        pyfp.close()


if __name__ == '__main__':
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    if len(sys.argv) == 1:
        call(["python", __file__, "OK"])
    elif sys.argv[1] == "OK":
        fimgs = glob.glob("*32x32.png")
        print fimgs
        for fimg in fimgs:
            fpy = fimg + ".py"
            if not os.path.exists(fpy):
                import wx
                import wx.tools.img2py
                # fimg = resize(fimg, 16, 16)
                imgName, ext = os.path.splitext(os.path.basename(fimg))
                imgName = imgName.replace('-', '__')
                wx.tools.img2py.img2py(fimg, fpy, imgName=imgName)
                write_index_category(fpy, fimg, imgName)
                if 'resized_' in fimg:
                    os.remove(fimg)

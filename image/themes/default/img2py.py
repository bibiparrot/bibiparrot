__author__ = 'shi'


import glob, os

import sys
from subprocess import call

if __name__ == '__main__':
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    if len(sys.argv) == 1:
        call(["python", __file__, "OK"])
    elif sys.argv[1] == "OK":
        fimgs = glob.glob("*.png")
        print fimgs
        for fimg in fimgs:
            fpy = fimg + ".py"
            if not os.path.exists(fpy):
                import wx
                import wx.tools.img2py
                wx.tools.img2py.img2py(fimg, fpy)
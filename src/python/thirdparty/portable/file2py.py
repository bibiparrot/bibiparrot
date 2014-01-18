################################################################################
# Name     : file2py.py                                                        #
# Brief    : read file and dump into python data object.                       #
#                                                                              #
# Url      :                                                                   #
#                                                                              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import glob, os
import sys
from subprocess import call

def filetopy(input_file, output_file):
    ### read  ###
    try:
        with open(input_file, 'rb') as fin:
            data = fin.read()
    finally:
        fin.close()
    ### write ###
    try:

        with open(output_file, 'w') as fout:
            fout.write('data = '+ repr(data))
    finally:
        fout.close()

###
##  http://en.wikipedia.org/wiki/Internet_media_type#Type_image
#
__ext_map__ = {
    '.gif':'image/gif',
    '.png':'image/png',
    '.jpg':'image/jpeg',
    '.svg':'image/svg+xml',
    '.flac':'audio/x-flac', ### http://en.wikipedia.org/wiki/FLAC ###
    '.mp3':'audio/mpeg',
    '.ogg':'audio/ogg',
    '.wav':'audio/wav',
}


if __name__ == '__main__':
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    if len(sys.argv) == 1:
        call(["python", __file__, "__OK__"])
    elif sys.argv[1] == "__OK__":
        types = ("*.gif", "*.png", "*.jpg", "*.txt", "*.tar.gz")
        ftyps = []
        for t in types:
            ftyps.extend(glob.glob(t))
            ftyps.extend(glob.glob(t.upper()))
        print ftyps
        for ftyp in ftyps:
            fpy = ftyp + ".py"
            if not os.path.exists(fpy):
                filetopy(ftyp, fpy)
            pass



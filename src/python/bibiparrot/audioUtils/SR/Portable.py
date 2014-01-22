################################################################################
# Name     : Portable.py                                                       #
# Brief    : load the portable directory into PATH and library (mac) into      #
#                 DYLD_FALLCALL_LIBRARY_PATH                                   #
#            provide a interface to call the commands                          #
#                                                                              #
# Url      :                                                                   #
#                                                                              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

import inspect, os, sys
import glob, os
import sys
from subprocess import call

__portables__ = []

###
##   <<<<<<<< PLEASE import needed portable modules here >>>>>>>>>>
#
#
def get_portables():
    # import portableflac
    # __portables__.append(portableflac)
    #
    # import portablexxx
    # __portables__.append(portablexxx)
    #
    return __portables__


###
##   output file into py module.
#
#
def file2py(input_file, output_file):
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
## tar -zxvf xxx.tar.gz
#
#
def tar_xfz(tarpath, xtrcpath=None):
    ### write .data
    if xtrcpath is None:
        xtrcpath = os.path.dirname(tarpath)
    import tarfile
    tar = tarfile.open(tarpath, "r:gz")
    try:
        tar.extractall(path=xtrcpath, members=tar)
    finally:
        tar.close()
    return xtrcpath

###
## tar -cxvf xxx.tar.gz xxx
#
#
def tar_cfz(fildir, tarpath=None, override=False):
    if tarpath is None:
        tarpath = os.path.basename(fildir) + '.tar.gz'
    if os.path.exists(tarpath) and not override:
        return tarpath
    import tarfile
    tar = tarfile.open(tarpath, "w")
    try:
        tar.add(fildir)
    finally:
        tar.close()
    return tarpath




###
##  extract a portable module (created by file2py.py)
#
#
def extract_portable_py(pymod, xtrcpath=None, override=False):
    ### write .data
    modfile = inspect.getfile(pymod)
    datapath = os.path.dirname(modfile)
    tarname = os.path.splitext(os.path.basename(modfile))[0] + '.tar.gz'
    if xtrcpath is None:
        xtrcpath = os.path.join(datapath, 'portable')
    if not os.path.exists(xtrcpath):
        os.mkdir(xtrcpath)
    tarpath = os.path.join(xtrcpath, tarname)
    if os.path.exists(tarpath) and not override:
        return xtrcpath
    fp = open(tarpath, 'wb')
    try:
        fp.write(pymod.data)
    finally:
        fp.close()
        ### remove the tar.gz file ###
        if override and os.path.exists(tarpath):
            os.remove(tarpath)
    xtrcpath = tar_xfz(tarpath, xtrcpath)
    return xtrcpath


def extract_all_py(xtrcpath=None, override=False):
    for mod in get_portables():
        xtrcpath = extract_portable_py(mod, xtrcpath, override)
    return xtrcpath

# tar_xfz(portable.tar.gz)
def extractall():
    curdir = os.path.dirname(inspect.getfile(inspect.currentframe()))
    potpath = os.path.join(curdir, 'portable.tar.gz')
    xtrpath = tar_xfz(potpath)
    return xtrpath

def append_sys_env(key, val):
    env = os.getenv(key, "")
    # if os.environ.has_key(key):
    #     env = os.getenv(key)
    env += os.pathsep + val
    os.environ[key] = env
    # print key,"=",env
    # print
    return env

###  ###

class Portable(object):
    IS_LOADED = False

    @staticmethod
    def load(prtdir = None):
        if prtdir is None:
            prtdir = os.path.dirname(inspect.getfile(inspect.currentframe()))
        # print "curdir=",curdir
        # prtdir = os.path.abspath(os.path.dirname(curdir))
        # print "prtdir=",prtdir

        ### http://stackoverflow.com/questions/973473/getting-a-list-of-all-subdirectories-in-the-current-directory ###
        for wlk in os.walk(prtdir):
            pkgdir = wlk[0]
            # print "pkgdir", os.path.abspath(pkgdir)
            pkgdirbase = os.path.basename(pkgdir).lower()
            # print pkgdirbase
            if pkgdirbase == 'win' or pkgdirbase == 'win32' or pkgdirbase == 'win64':
                # print '<----------------'
                pkgdir = os.path.abspath(pkgdir)
                # sys.path.append(pkgdir)
                append_sys_env('PATH', pkgdir)
                # print '----------------->'
            ### for win or mac os ###
            elif pkgdirbase == 'mac' or pkgdirbase == 'mac32' or pkgdirbase == 'mac64':
                # print '<----------------'
                pkgdir = os.path.abspath(pkgdir)
                # sys.path.append(pkgdir)
                append_sys_env('PATH', pkgdir)
                # print '----------------->'
                ### add library path (mac ox), set env DYLD_FALLBACK_LIBRARY_PATH ###
                append_sys_env("DYLD_FALLBACK_LIBRARY_PATH", pkgdir)
                # print os.getenv("DYLD_FALLBACK_LIBRARY_PATH")

    @staticmethod
    def call (pbin, *args, **kwargs):
        ### load library before call ###
        if not Portable.IS_LOADED:
            portablepath = extractall()
            Portable.load(portablepath)
            Portable.IS_LOADED = True
        ### prepare the parameters ###
        # print 'args =', args
        params = [str(pbin)]
        ### all parameters must be string  ###
        for arg in args:
            params.append(str(arg))
        # print 'params =', ' '.join(params)
        import subprocess
        return subprocess.call(params)

### @End class Portable


def file2py_by_types(dir='.', types=("*.gif", "*.png", "*.jpg", "*.txt", "*.tar.gz")):
    ftyps = []
    for t in types:
        ftyps.extend(glob.glob(os.path.join(dir, t)))
        ftyps.extend(glob.glob(os.path.join(dir, t.upper())))
    print ftyps
    for ftyp in ftyps:
        fpy = ftyp + ".py"
        if not os.path.exists(fpy):
            file2py(ftyp, fpy)


if __name__ == '__main__':
    os.environ["VERSIONER_PYTHON_PREFER_32_BIT"] = "yes"
    if len(sys.argv) == 1:
        call(["python", __file__, "__OK__"])
    elif sys.argv[1] == "__OK__":
        file2py_by_types()
        pass



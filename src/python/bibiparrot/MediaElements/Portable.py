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

try:
    from ...bibiparrot.Constants.constants import __default_portable_gzip__
except (ValueError, ImportError) as err:
    __default_portable_gzip__ = 'portable.tar.gz'


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
def tar_xfz(tarpath, xtrcpath=None, override=False):
    ### write .data
    if xtrcpath is None:
        tarname = os.path.basename(tarpath)
        tarname = tarname.replace(r'.tar.gz', '')
        tarname = tarname.replace(r'.tgz', '')
        xtrcpath = os.path.join(os.path.dirname(tarpath), tarname)
    import tarfile
    if override or not os.path.exists(xtrcpath):
        print 'extract', tarpath, 'into', xtrcpath, '...'
        tar = tarfile.open(tarpath, "r:gz")
        try:
            tar.extractall(path=xtrcpath, members=tar)
        finally:
            tar.close()
        print 'extract end.'
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
    potpath = os.path.join(curdir, __default_portable_gzip__)
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

###
##  Dummy class to organize functions
#
class Portable(object):
    __slots__ = []

    LOADED = ''
    @staticmethod
    def loadlib(prtdir = None):
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
    def load():
        ### load only once. ###
        # print 'load', Portable.LOADED, os.path.exists(Portable.LOADED)
        if not Portable.LOADED or not os.path.exists(Portable.LOADED):
            portablepath = extractall()
            # print 'portablepath', portablepath
            Portable.loadlib(portablepath)
            Portable.LOADED = portablepath
            print 'Portable.LOADED', Portable.LOADED
        return Portable.LOADED

    @staticmethod
    def call (pbin, *args, **kwargs):
        ### load library before call ###
        Portable.load()
        ### prepare the parameters ###
        # print 'args =', args
        params = [str(pbin)]
        ### all parameters must be string  ###
        for arg in args:
            params.append(str(arg))
        # print 'params =', ' '.join(params)
        import subprocess
        return subprocess.call(params)

    @staticmethod
    def find(nam, typ='f', mrk=''):
        ### load library before call ###
        prtdir = Portable.load()
        idx = 2 # default search files#
        if typ == 'd':
            idx = 1 # search directory #
        if os.path.exists(prtdir):
            # print 'prtdir', prtdir
            for wlk in os.walk(prtdir):
                for tgt in wlk[idx]:
                    # print 'tgt', tgt
                    if nam == tgt:
                        tgtpath = os.path.abspath(os.path.join(wlk[0], tgt))
                        # print 'tgtpath', tgtpath
                        if mrk in tgtpath:
                            return tgtpath
        # not found #
        return None


### @End class Portable

###
##  load portable/vlc-*/libs
#

def load_portable_vlc():
    dll = None
    plugin_path = None
    try:
        import ctypes
        print 'begin ...'
        if sys.platform.startswith('linux'):
            p = Portable.find('libvlc.so.5')
            # print 'Portable.load',p
            if p is not None:
                try:
                    dll = ctypes.CDLL(p)
                except OSError:  # may fail
                    dll = ctypes.CDLL('libvlc.so.5')
        elif sys.platform.startswith('win'):
            p = Portable.find('libvlc.dll')
            if p is not None:  # try loading
                    ### libvlc.dll ###
                    cwd = os.getcwd()
                    os.chdir(os.path.dirname(p))
                     # if chdir failed, this will raise an exception
                    dll = ctypes.CDLL('libvlc.dll')
                     # restore cwd after dll has been loaded
                    os.chdir(cwd)
                    ### ./plugins ###
                    plugin_path = os.path.join(os.path.dirname(p), 'plugins')
            else:  # may fail
                    dll = ctypes.CDLL('libvlc.dll')
        elif sys.platform.startswith('darwin'):
            ### lib/libvlc.dylib ###
            p = Portable.find('libvlc.dylib')
            print 'Portable.load',p
            if p is not None:
                dll = ctypes.CDLL(p)
                ### lib/../plugins ###
                plugin_path = os.path.join(os.path.dirname(os.path.dirname(p)), 'plugins')
            else:  # hope, some PATH is set...
                dll = ctypes.CDLL('libvlc.dylib')
    except ImportError as err:
        print 'load_portable_vlc error', err
        pass
    print 'dll', dll
    print 'plugin', plugin_path
    return (dll, plugin_path)


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



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


def append_sys_env(key, val):
    env = os.getenv(key, "")
    # if os.environ.has_key(key):
    #     env = os.getenv(key)
    env += os.pathsep + val
    os.environ[key] = env
    # print key,"=",env
    # print
    return env

###
##  tar cvfz - flac-1.2.1/ sox-14.4.1/ vlc-2.0.8/ | split -b 10m  - portable.tar.gz.
#

def split_into_chunks(gzipf='portable.tar.gz', max_size  = 10 * 1024 * 1024, buf_size  = 100 * 1024 * 1024):
    # 'portable.tar.gz' - default file name
    # 500Mb  - max chapter size
    # 50GB   - memory buffer size

    chunks = 0
    uglybuf  = ''
    with open(gzipf, 'rb') as gzipfp:
      while True:
        tgtfp = open(gzipf+'.%02d' % chunks, 'w')
        written = 0
        try:
            while written < max_size:
              tgtfp.write(uglybuf)
              tgtfp.write(gzipfp.read(min(buf_size, max_size-written)))
              written += min(buf_size, max_size-written)
              uglybuf = gzipfp.read(1)
              if len(uglybuf) == 0:
                break
        finally:
            tgtfp.close()
        if len(uglybuf) == 0:
          break
        chunks += 1

class Portable(object):
    IS_LOADED = False

    @staticmethod
    def load():
        prtdir = os.path.abspath(os.path.dirname(inspect.getfile(inspect.currentframe())))
        # print "curdir=",curdir
        # prtdir = os.path.abspath(os.path.dirname(curdir))
        # print "prtdir=",prtdir
        ### http://stackoverflow.com/questions/973473/getting-a-list-of-all-subdirectories-in-the-current-directory ###
        for wlk in os.walk(prtdir).next()[1]:
            pkgdir = os.path.join(prtdir, wlk)
            ### for mac os ###
            # sys.path.append(os.path.join(pkgdir, 'mac'))
            append_sys_env("PATH", os.path.join(pkgdir, 'mac'))
            ### for win os ###
            # sys.path.append(os.path.join(pkgdir, 'win'))
            append_sys_env("PATH", os.path.join(pkgdir, 'win'))
            ### add library path (mac ox), set env DYLD_FALLBACK_LIBRARY_PATH ###
            append_sys_env("DYLD_FALLBACK_LIBRARY_PATH", os.path.join(pkgdir, 'mac'))
            # print os.getenv("DYLD_FALLBACK_LIBRARY_PATH")


    @staticmethod
    def call (pbin, *args):
        ### load library before call ###
        if not Portable.IS_LOADED:
            Portable.load()
            Portable.IS_LOADED = True
        ### prepare the parameters ###
        params = [pbin]
        params.extend(args)
        import subprocess
        return subprocess.call(params)

### @End class Portable

if __name__ == '__main__':
    import os, inspect
    curdir = os.path.abspath(os.path.dirname(inspect.getfile(inspect.currentframe())))
    print 'curdir', curdir
    cmds = "cd "+curdir+"; tar cvfz portable.tar.gz"

    for subdir in os.walk(curdir):
        #['flac-1.2.1','sox-14.4.1','vlc-2.0.8']:
        for i in subdir[1]:
            cmds = cmds +" "+ i;
        break
    print cmds
    import subprocess as subp
    subp.check_call(str(cmds), shell=True)
    # from subprocess import call
    # call(["ls", "-l"])

    split_into_chunks(os.path.join(curdir,'portable.tar.gz'))
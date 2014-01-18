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
            sys.path.append(os.path.join(pkgdir, 'mac'))
            ### for win os ###
            sys.path.append(os.path.join(pkgdir, 'win'))
            ### add library path (mac ox), set env DYLD_FALLBACK_LIBRARY_PATH ###
            append_sys_env("DYLD_FALLBACK_LIBRARY_PATH", os.path.join(pkgdir, 'mac'))
            print os.getenv("DYLD_FALLBACK_LIBRARY_PATH")


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


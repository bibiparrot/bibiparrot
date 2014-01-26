################################################################################
# Name     : utils.py                                                          #
# Brief    : Import the required base modules needed for launching Bibiparrot  #
#                                                                              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################




import sys, os , time, inspect, imp, platform, logging, traceback
import wx
import ConfigParser

from ...bibiparrot.Constants.constants import __default_size_splitter__

##
# @url: http://stackoverflow.com/questions/1095543/get-name-of-calling-functions-module-in-python
##
@DeprecationWarning
def directory():
    filename = os.path.abspath(inspect.getfile(inspect.stack()[1][0]))
    path = os.path.dirname(filename)
    return path

### only allowed in log ###
def funcname():
    stack = traceback.extract_stack()
    filename, lineno, name, text = stack[-2]
    return name


__urllib_imported__ = False
def path2url(path):
    """Return file:// URL from a filename."""
    path = os.path.abspath(path)
    if isinstance(path, unicode):
        path = path.encode('utf8')
    global __urllib_imported__
    if not __urllib_imported__:
        import urllib
        __urllib_imported__ = True
    return 'file://' + urllib.pathname2url(path)

def split2array(str, sep):
    arr = []
    if not str is None and not str.strip() == "":
        for s in str.strip().split(sep):
            arr.append(s.strip())
    return arr


def split2intarray(str, sep):
    res = []
    if not str == None and not str.strip() == "" and \
       not str == None and not str.strip() == "" and \
       sep in str:
        for item in str.strip().split(sep):
            res.append(int(item))
    return res


def split2pair(str, sep):
    if sep in str:
        pair = str.split(sep)
        return (pair[0].strip(), pair[1].strip())
    else:
        return ()


def str2dim(str):
    res = (0, 0)
    if __default_size_splitter__ in str:
        arr = split2intarray(str.lower(), __default_size_splitter__)
        if len(arr) == 2:
            res = (arr[0], arr[1])
    return res

# def str2long(str):
#     if str is None or str.strip == "":
#         return None
#     else:
#         str = str.replace(",", "")
#         return long(str)

def str2long(str):
    try:
        return long(str)
    except TypeError:
        return None

def str2int(str):
    if str is None or str.strip == "":
        return None
    else:
        str = str.replace(",", "")
        return int(str)

# def str2bool(str):
#     if str is None or type(str) == type(""):
#         return None
#     else:
#         if str.strip().lower() in ("yes", "y", "true",  "t", "1"):
#             return True
#         elif str.strip().lower() in ("no",  "n", "false", "f", "0", ""):
#             return False
#         else:
#             raise Exception('Invalid value for boolean: ' + str)


def str2bool(str):
    try:
        return str.lower() in ("yes", "y", "true",  "t", "1")
    except Exception as err:
        # raise Exception('Invalid value for boolean: ' + err)
        return False


class Bean(object):
    def __init__(self,**kwargs):
        self.clear()
        if kwargs is not None:
            for item in kwargs.keys():
                setattr(self, item, kwargs[item])

    def clear(self):
        pass

    def asDict(self):
        result={}
        for item in self.__slots__:
            # if hasattr(self,item):
            #     result[item] = getattr(self,item)
            result[item] = getattr(self, item, None)
        return result

    @DeprecationWarning
    def hasAttr(self, name):
        return hasattr(self, name)

    def getAttr(self, name, default):
        return getattr(self, name, default)

    def dump(self):
        result = []
        dict = self.asDict()
        for n in dict.iterkeys():
             result.append("%s = %s" % (n, dict.get(n)))
        return "{\n\t\t"+"\n\t\t".join(result)+"\n}"


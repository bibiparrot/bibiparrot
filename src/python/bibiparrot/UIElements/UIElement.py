
import sys, os , time, inspect, imp, platform, logging
import wx

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import __default_size_splitter__

# Name=MainFrame
# Title=Bibiparrot
# Type=MainFrame
# Icon=
# Position=20x20
# Size=800x600
# Layer=0



def split2dim(str):
    res = (0, 0)
    arr = split2intarray(str, __default_size_splitter__)
    if len(arr) == 2:
        res = (arr[0], arr[1])
    return res


class UIElement(object):
    __slots__=("__conf__", "dataFunc", "Section", "Name", "Title", "Type", "Icon", "Position", "Size", "Layer", "Data")
    def __init__(self):
        self.__conf__ = uiconf()
        self.dataFunc = None
        self.clear()

    def clear(self):
        self.Section = None
        self.Name = ""
        self.Title = ""
        self.Section = ""
        self.Type = ""
        self.Icon = ""
        self.Position = (0,0)
        self.Size = (0,0)
        self.Layer = 0
        self.Data = []


    def asDict(self):
        result={}
        for item in self.__slots__:
            result[item] = getattr(self,item)
        return result

    def dump(self):
        result = []
        # for n, v in sorted(self.asDict()):
        #     result.append("%s = %s" % (n, v))
        dict = self.asDict()
        for n in dict.iterkeys():
             result.append("%s = %s" % (n, dict.get(n)))
        return "{\n\t\t"+"\n\t\t".join(result)+"\n}"

    def loadSect(self, section):
        self.clear()
        self.Section = section
        if LOGWIRE:
            log().debug("%s: section=%s, dataFunc=%s",funcname(), self.Section, self.dataFunc)
        for key in self.__slots__:
            val = self.__conf__.getConf(section, key)
            if not val is None and not val == "":
                if key in ["Position", "Size"]:
                    setattr(self, key, split2dim(val))
                elif key in ["Layer"]:
                    setattr(self, key, int(val))
                elif key in ["Data"] and not self.dataFunc is None:
                    setattr(self, key, self.dataFunc(self, val))
                elif key in ["Name", "Title", "Type", "Icon"]:
                    setattr(self, key, val)
        return self


def dataFunc(uie, val):
    return []
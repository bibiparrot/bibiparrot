
from ...bibiparrot.Constants.constants import __required_wx_version__, \
    __default_size_splitter__, __default_size_splitter__, __default_ui_element_sep__


import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *


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
    def __init__(self, dataFunc=None):
        self.__conf__ = uiconf('UIElement')
        self.dataFunc = dataFunc
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
        self.Data = None


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
                elif key in ["Data"]:
                    if not self.dataFunc is None:
                        setattr(self, key, self.dataFunc(self, val))
                    else:
                        setattr(self, key, val)
                elif key in ["Name", "Title", "Type", "Icon"]:
                    setattr(self, key, val)
        return self


def dataFunc(uie, val):
    return []

import sys, os , time, inspect, imp, platform, logging
import wx

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import __default__size__splitter__

# Name=MainFrame
# Title=Bibiparrot
# Type=MainFrame
# Icon=
# Position=20x20
# Size=800x600
# Layer=0



def split2dim(str):
    res = (0, 0)
    arr = split2intarray(str, __default__size__splitter__)
    if len(arr) == 2:
        res = (arr[0], arr[1])
    return res

class UIElement(object):
    __slots__=("Name", "Title", "Type", "Icon", "Position", "Size", "Layer")
    def __init__(self):
        self.clear()

    def clear(self):
        self.Name = ""
        self.Title = ""
        self.Type = ""
        self.Icon = ""
        self.Position = (0,0)
        self.Size = (0,0)
        self.Layer = 0

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
        self.clear();
        for key in self.__slots__:
            val = uiconf().getConf(section, key)
            if not val == None and not val == "":
                if key == "Position" or key == "Size":
                    setattr(self, key, split2dim(val))
                elif key == "Layer":
                    setattr(self, key, int(val))
                else:
                    setattr(self, key, val)
        return self
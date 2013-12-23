

import sys, os , time, inspect, imp, platform, logging
import wx

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import *


class MainFrame(wx.Frame):
    def __init__(self, parent, title):
        title = uiconf().getConf("MainFrame", "Title")
        if LOGWIRE:
            log().debug("%s: title = %s", funcname(), title)
        # print title
        wx.Frame.__init__(self, parent, title=title, size=(200,100))
        self.control = wx.TextCtrl(self, style=wx.TE_MULTILINE)
        self.Show(True)


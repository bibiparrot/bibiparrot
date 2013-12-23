

import sys, os , time, inspect, imp, platform, logging
import wx

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.UIElements.UIElement import *

class MainFrame(wx.Frame):
    def __init__(self, parent):
        self.element =  UIElement().loadSect("MainFrame")
        if LOGWIRE:
            log().debug("%s: title = %s", funcname(), self.element.Title)
        print unicode(self.element.Title).encode('utf8')
        wx.Frame.__init__(self, parent, title=self.element.Title, size=(200,100))
        self.control = wx.TextCtrl(self, style=wx.TE_MULTILINE)
        self.Show(True)


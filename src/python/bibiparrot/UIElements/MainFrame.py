################################################################################
# Name     : MainFrame.py                                                      #
# Brief    : Top level window as the frame for bibiparrot windows .            #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.Frame-class.html              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################


import sys, os , time, inspect, imp, platform, logging
import wx
import wx.grid
import wx.html
import wx.aui

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.UIElements.UIElement import UIElement

class MainFrame(wx.Frame):
    def __init__(self, parent):
        self.element =  UIElement().loadSect("MainFrame")
        if LOGWIRE:
            log().debug("%s: title = %s", funcname(), self.element.Title)
        # print unicode(self.element.Title).encode('utf8')
        wx.Frame.__init__(self, parent, title=self.element.Title, size=self.element.Size, pos = self.element.Position)
        # self.uiman = wx.aui.AuiManager(self)
        self.control = wx.TextCtrl(self, style=wx.TE_MULTILINE)
        # self.Show(True)


################################################################################
# Name     : MainFrame.py                                                      #
# Brief    : Top level window as the frame for bibiparrot windows .            #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.Frame-class.html              #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################



from ...bibiparrot.Constants.constants import __required_wx_version__

import sys, os , time, inspect, imp, platform, logging
import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.grid
import wx.html
import wx.aui

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.UIElements.UIElement import UIElement


from ..Configurations import configurations
from ..Constants import constants



class MainFrame(wx.Frame):
    def __init__(self, parent=None):
        self.element =  UIElement().loadSect("MainFrame")
        if LOGWIRE:
            log().debug("__init__: title = %s", self.element.Title)
        # print unicode(self.element.Title).encode('utf8')
        wx.Frame.__init__(self, parent, title=self.element.Title, size=self.element.Size, pos = self.element.Position,
                          style=wx.DEFAULT_FRAME_STYLE |
                                            wx.SUNKEN_BORDER |
                                            wx.CLIP_CHILDREN)
        # self.uiman = wx.aui.AuiManager()
        # self.uiman.SetManagedWindow(self)
        # self.uiman.Update()
        # self.control = wx.TextCtrl(self, style=wx.TE_MULTILINE)
        # wx.Frame.SetStatusBar
        # self.Show(True)



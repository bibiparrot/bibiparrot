################################################################################
# Name     : Repeater.py                                                       #
# Brief    : The Panel to hold repeater controller.                            #
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
import wx.richtext

from ..Configurations import configurations
from ..Constants import constants
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.UIElements.EditControl import EditControl
from ...bibiparrot.UIElements.MainMenu import MainMenu
from ...bibiparrot.UIElements.MainToolbar import  RepeaterToolbar

from ...bibiparrot.Configurations.configurations import *

class Repeater(wx.Panel):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("Repeater")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.Toolbar = RepeaterToolbar(self)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self)
        # self.uiman.AddPane(self.control, wx.aui.AuiPaneInfo().
        #           CenterPane())
        self.uiman.AddPane(self.Toolbar, wx.aui.AuiPaneInfo().
                  Name("RepeaterToolbar").Caption("RepeaterToolbar").
                  ToolbarPane().Top().Row(1).Position(1).
                  LeftDockable(False).RightDockable(False).CloseButton(False))
        self.uiman.Update()






################################################################################
# Name     : MainTabs.py                                                       #
# Brief    : Set tablets .                                                     #
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
from ...bibiparrot.UIElements.MainToolbar import MainToolbar, EditorToolbar

from ...bibiparrot.Configurations.configurations import *
#
class Tabs(wx.aui.AuiNotebook):
    def __init__(self, parent, *args, **kwargs):
        wx.aui.AuiNotebook.__init__(self, parent, *args, **kwargs)
        pass

    def addTab(self, tab, *args, **kwargs):
        self.AddPage(tab, tab.Name, *args, **kwargs)

class DirCtrl(wx.GenericDirCtrl):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("DirCtrl")
        wx.GenericDirCtrl.__init__(self, parent,
                                   name = self.element.Title,
                                   style=wx.DIRCTRL_SHOW_FILTERS,
                                   filter=self.element.Data,
                                   *args, **kwargs)
        pass

class MainTabs(wx.Panel):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MainTabs")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self)
        self.tabs = Tabs(self)
        self.dirctrl = DirCtrl(self.tabs)
        self.tabs.addTab(self.dirctrl)
        self.uiman.AddPane(self.tabs, wx.aui.AuiPaneInfo().
                          CenterPane())
        self.uiman.Update()
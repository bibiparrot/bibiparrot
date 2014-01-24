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
import wx.dataview
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
        self.AddPage(tab, tab.element.Title, *args, **kwargs)

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


def dataFuncFileList(ele, val):
    data = []
    sec = ele.Section
    if not val is None and not val.strip() == "":
        for tool in split2array(val, constants.__default_ui_element_sep__):
            subsec = sec + "." + tool
            if LOGWIRE:
                log().debug("%s: subsec=%s, conf=%s", funcname(), subsec, ele.__conf__)
            data.append(FileListColumnBean(subsec, ele.__conf__, dataFuncFileList))
    return data


class FileListColumnBean(Bean):
    __slots__ = ["__conf__", "Section", "Size", "Enabled", "Name", "Label", "Icon", "Help", "More"]
    def __init__(self, sec, conf, func):
        self.Section = sec
        self.__conf__ = conf
        for key in self.__slots__:
            val = conf.getConf(self.Section, key)
            if LOGWIRE:
                log().debug("%s: section=%s, %s=%s", funcname(), self.Section, key, val)

            ###  default value is required ###
            if key in ["Enabled"]:
                setattr(self, key, False)

            if not val is None and not val == "":
                if key in ["More"] and not func is None:
                    setattr(self, key, func(self, val))
                elif key in ["Name", "Label", "Help", "Style", "Icon", "Id"]:
                    setattr(self, key, val)
                elif key in ["Position", "Size"]:
                    setattr(self, key, str2dim(val))
                elif key in ["Enabled"]:
                    setattr(self, key, str2bool(val))
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.dump())




class FileListCtrl(wx.dataview.DataViewListCtrl):
    def __init__(self, parent, *args, **kwargs):
        wx.dataview.DataViewListCtrl.__init__(self, parent, *args, **kwargs)
        self.element = UIElement(dataFuncFileList)
        self.element.loadSect("FileListCtrl")
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.element.dump())
        for bean in self.element.Data:
            self.AppendTextColumn(bean.Label, width=bean.Size[0])



class MainTabs(wx.Panel):
    def __init__(self, parent, *args, **kwargs):
        self.element = UIElement()
        self.element.loadSect("MainTabs")
        wx.Panel.__init__(self, parent, size = self.element.Size, *args, **kwargs)
        self.uiman = wx.aui.AuiManager()
        self.uiman.SetManagedWindow(self)
        self.tabs = Tabs(self)
        self.DirCtrl = DirCtrl(self.tabs)
        self.FileListCtrl = FileListCtrl(self.tabs)
        self.tabs.addTab(self.DirCtrl)
        self.tabs.addTab(self.FileListCtrl)
        self.uiman.AddPane(self.tabs, wx.aui.AuiPaneInfo().
                          CenterPane())
        self.uiman.Update()
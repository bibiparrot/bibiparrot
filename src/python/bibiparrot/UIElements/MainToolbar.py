################################################################################
# Name     : MainToolbar.py                                                    #
# Brief    : #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.ToolBar-class.html            #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

import wxversion
wxversion.select('2.8-unicode')

import sys, os , time, inspect, imp, platform, logging
import wx
import wx.grid
import wx.html
import wx.aui

from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import __default_ui_element_sep__
from ...bibiparrot.UIElements.UIElement import UIElement

from ...bibiparrot import images

def dataFuncToolbar(ele, val):
    data = []
    sec = ele.Section
    if not val is None and not val.strip() == "":
        for tool in split2array(val, __default_ui_element_sep__):
            subsec = sec + "." + tool
            if LOGWIRE:
                log().debug("%s: subsec=%s, conf=%s", funcname(), subsec, ele.__conf__)
            data.append(ToolbarBean(subsec, ele.__conf__, dataFuncToolbar))
    return data


class ToolbarBean(Bean):
    __slots__ = ["__conf__", "Section", "Id", "Position", "Size", "Style", "Enabled", "Name", "Icon", "Help", "Data"]
    def __init__(self, sec, conf, func):
        self.Section = sec
        self.__conf__ = conf
        for key in self.__slots__:
            val = conf.getConf(self.Section, key)
            if LOGWIRE:
                log().debug("%s: section=%s, %s=%s", funcname(), self.Section, key, val)
            if not val is None and not val == "":
                if key in ["Data"] and not func is None:
                    setattr(self, key, func(self, val))
                elif key in ["Name", "Help", "Style"]:
                    setattr(self, key, val)
                elif key in ["Id"] and not val is None:
                    setattr(self, key, str2int(val))
                elif key in ["Position"]:
                    setattr(self, key, str2dim(val))
                elif key in ["Enabled"]:
                    setattr(self, key, str2bool(val))

            # elif key in ["Texts"]:
            #     setattr(self, key, split2array(val, __default_ui_element_sep__))
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.dump())

    pass


class MainToolbar (wx.ToolBar):
    def __init__(self, parent):
        self.element = UIElement()
        self.element.dataFunc = dataFuncToolbar
        self.element.loadSect("MainToolbar")

        if LOGWIRE:
            log().debug("%s: element=%s", funcname(), self.element.dump())
        wx.ToolBar.__init__(self, parent)
        for toolbar in self.element.Data:
            if LOGWIRE:
                log().debug("%s: menu=%s", funcname(), toolbar.dump())
            # self.Append(Menu(menu), menu.Title)
            # wx.ToolBar.AddTool(self, toolbar.Id)
            wx.ToolBar.AddLabelTool(self, toolbar.Id, label=toolbar.Name, bitmap=images.WXPdemo.GetBitmap())
            if toolbar.hasAttr("Data"):
                for subToolbar in toolbar.Data:
                    wx.ToolBar.AddLabelTool(self, subToolbar.Id, label=subToolbar.Name,
                                            bitmap=images.WXPdemo.GetBitmap())
            wx.ToolBar.AddSeparator(self)

        # wx.ToolBar.SetMargins(self, (20, 20))
        wx.ToolBar.Realize(self)

        # parent.SetToolBar(self)
        # tbar = self.CreateToolBar()
        # doBind( tbar.AddTool(-1, images._rt_open.GetBitmap(),
        #                     shortHelpString="Open"), self.OnFileOpen)
        # doBind( tbar.AddTool(-1, images._rt_save.GetBitmap(),
        #                     shortHelpString="Save"), self.OnFileSave)
        # tbar.AddSeparator()
        # tbar.Realize()
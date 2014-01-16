################################################################################
# Name     : MainToolbar.py                                                    #
# Brief    : Define the necessary Toolbar for Bibi Parrot project              #
#                                                                              #
# Url      : http://www.wxpython.org/docs/api/wx.ToolBar-class.html            #
# Author   : Chunqi SHI <diligence.cs@gmail.com>                               #
# Copyright: &copy 2013 ~ present Chunqi SHI   <diligence.cs@gmail.com>        #
################################################################################

from ...bibiparrot.Constants.constants \
    import __required_wx_version__,  __default_ui_element_sep__


import sys, os , time, inspect, imp, platform, logging

import wxversion
wxversion.select(__required_wx_version__)
import wx
import wx.grid
import wx.html
import wx.aui


from ..Configurations import configurations
from ..Constants import constants

from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.UIElements.UIElement import UIElement
# from ...bibiparrot import images
from Images import bitmap
from EventIDs import EventIDs
from SelfControls import SearchCtrl


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
    __slots__ = ["__conf__", "Section", "Id", "Position", "Size", "Style", "Enabled", "Name", "Label", "Icon", "Help", "More"]
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
                elif key in ["Name", "Label", "Help", "Style", "Icon"]:
                    setattr(self, key, val)
                elif key in ["Id"] and not val is None:
                    setattr(self, key, str2long(val))
                elif key in ["Position"]:
                    setattr(self, key, str2dim(val))
                elif key in ["Enabled"]:
                    setattr(self, key, str2bool(val))

            # elif key in ["Texts"]:
            #     setattr(self, key, split2array(val, __default_ui_element_sep__))
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.dump())

    def hasMore(self):
        return self.hasAttr("More")

    def hasSeparator(self):
        return self.hasAttr("Style") and "separator" in self.Style.lower()

    def isSearchCtrl(self):
        return self.hasAttr("Style") and "searchctrl" in self.Style.lower()

    def needsUpdate(self):
        return self.hasAttr("Style") and "update" in self.Style.lower()

### @End class ToolbarBean

class Toolbar (wx.ToolBar):
    def __init__(self, parent, sect, *args, **kwargs):
        ### {id:(toolbar, wxToolbar)}###
        self.binds = {}
        self.element = UIElement()
        self.element.dataFunc = dataFuncToolbar
        self.element.loadSect(sect)
        if LOGWIRE:
            log().debug("%s: element=%s", funcname(), self.element.dump())
        wx.ToolBar.__init__(self, parent, *args, **kwargs)
        # self.SetToolBitmapSize(self.element.Size)

        for toolbar in self.element.Data:
            if LOGWIRE:
                log().debug("%s: menu=%s", funcname(), toolbar.dump())
            self.add(toolbar)
        ###  add space to toolbar ###
        # wx.ToolBar.AddLabelTool(self, -1, label="END", bitmap=wx.NullBitmap)
        wx.ToolBar.Realize(self)

    def add(self, toolbar):
        if toolbar.isSearchCtrl():
            if selfctrls.has_key(toolbar.Name):
                wx.ToolBar.AddControl(self, selfctrls[toolbar.Name](self))
        else:
            if toolbar.Enabled:
                id = EventIDs.getID(toolbar)
                # print toolbar.Name
                item = wx.ToolBar.AddTool(self, id, bitmap(toolbar.Icon), isToggle=toolbar.needsUpdate())
                self.binds[id] = (toolbar, item)
            if toolbar.hasMore():
                for subToolbar in toolbar.More:
                    self.add(subToolbar)

        if toolbar.hasSeparator():
            wx.ToolBar.AddSeparator(self)

### @End class Toolbar


        # if hasattr(parent, "mgr"):
        #     parent.mgr.AddPane(self, wx.aui.AuiPaneInfo().
        #                   Name("Toolbar").Caption("Toolbar").
        #                   ToolbarPane().Top().Row(1).Position(1).
        #                   LeftDockable(False).RightDockable(False))
        #
        # parent.SetToolBar(self)
        # tbar = self.CreateToolBar()
        # doBind( tbar.AddTool(-1, images._rt_open.GetBitmap(),
        #                     shortHelpString="Open"), self.OnFileOpen)
        # doBind( tbar.AddTool(-1, images._rt_save.GetBitmap(),
        #                     shortHelpString="Save"), self.OnFileSave)
        # tbar.AddSeparator()
        # tbar.Realize()


class MainToolbar (Toolbar):
    def __init__(self, parent, *args, **kwargs):
        Toolbar.__init__(self, parent, sect="MainToolbar", *args, **kwargs)

class EditorToolbar (Toolbar):
    def __init__(self, parent, *args, **kwargs):
        Toolbar.__init__(self, parent, sect="EditorToolbar", *args, **kwargs)
### @End class MainToolbar
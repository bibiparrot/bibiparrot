

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


def dataFuncMenu(ele, val):
    data = []
    sec = ele.Section
    if not val is None and not val.strip() == "":
        for menu in split2array(val, __default_ui_element_sep__):
            subsec = sec + "." + menu
            if LOGWIRE:
                log().debug("%s: subsec=%s, conf=%s", funcname(), subsec, ele.__conf__)
            data.append(MenuBean(subsec, ele.__conf__, dataFuncMenu))
    return data


class MenuBean(Bean):
    __slots__ = ["__conf__", "Section", "Id", "Title", "Text", "Help", "Kind", "Data"]
    def __init__(self, sec, conf, func):
        self.Section = sec
        self.__conf__ = conf
        for key in self.__slots__:
            val = conf.getConf(self.Section, key)
            if LOGWIRE:
                log().debug("%s: %s=%s", funcname(), key, val)
            if key in ["Data"] and not func is None:
                setattr(self, key, func(self, val))
            elif key in ["Id", "Title", "Help", "Kind","Text"]:
                setattr(self, key, val)
            # elif key in ["Texts"]:
            #     setattr(self, key, split2array(val, __default_ui_element_sep__))
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.dump())

    pass

class Menu(wx.Menu):
    def __init__(self, bean):
        self.MenuBean = bean
        if LOGWIRE:
            log().debug("%s: MenuBean=%s", funcname(), self.MenuBean.dump())
        assert len(self.MenuBean.Texts) > 0
        wx.Menu.__init__(self, self.MenuBean.Texts[0])

        if len(self.MenuBean.Texts) > 1:
            for more in self.MenuBean.Texts[1:]:
               wx.Menu.Append(self, 1, text=more)
    pass


class MainMenu(wx.MenuBar):
    def __init__(self, parent):
        self.element = UIElement()
        self.element.dataFunc = dataFuncMenu
        self.element.loadSect("MainMenu")
        if LOGWIRE:
            log().debug("%s: element=%s", funcname(), self.element.dump())
        wx.MenuBar.__init__(self)
        for menu in self.element.Data:
            if LOGWIRE:
                log().debug("%s: menu=%s", funcname(), menu.dump())
            self.Append(Menu(menu), menu.Title)
        parent.SetMenuBar(self)




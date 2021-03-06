################################################################################
# Name     : MainMenu.py                                                       #
# Brief    : Read menu from ui.cfg and display the menus.                      #
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

from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.Configurations.configurations import *
from ...bibiparrot.Constants.constants import __default_ui_element_sep__
from EventIDs import getIDbyElement
# from ..Configurations import configurations
# from ..Constants import constants



__default_menu_type_separator__ = "Separator"
__default_menu_type_radioitem__ = "RadioItem"
__default_menu_type_item__ = "Item"


def withShortCut(title, shortcut):
    ret = "&" + title
    if shortcut is not None and  not shortcut.strip() == "":
        ret = ret + "\t" + shortcut
    return ret


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
    __slots__ = ["__conf__", "Section", "Id", "Title", "Text", "Shortcut", "Help", "Kind", "Data"]
    def __init__(self, sec, conf, func):
        self.Section = sec
        self.__conf__ = conf
        for key in self.__slots__:
            val = conf.getConf(self.Section, key)
            if LOGWIRE:
                log().debug("%s: %s=%s", funcname(), key, val)
            if key in ["Data"] and not func is None:
                setattr(self, key, func(self, val))
            elif key in ["Title", "Shortcut", "Help", "Kind","Text","Id"]:
                setattr(self, key, val)
            # elif key in ["Id"] and not val is None:
            #     setattr(self, key, str2int(val))
            # elif key in ["Texts"]:
            #     setattr(self, key, split2array(val, __default_ui_element_sep__))

        ### replace id ###
        self.Id = getIDbyElement(self)
        if LOGWIRE:
            log().debug("%s: Bean=%s", funcname(), self.dump())

    pass


class Menu(wx.Menu):
    def __init__(self, bean):
        self.MenuBean = bean
        if LOGWIRE:
            log().debug("%s: MenuBean=%s", funcname(), self.MenuBean.dump())

        ''' Each Menu Must Has Items '''
        assert len(self.MenuBean.Data) > 0
        wx.Menu.__init__(self)
        for subMenuBean in self.MenuBean.Data:
            if subMenuBean.Kind == __default_menu_type_separator__:
                wx.Menu.AppendSeparator(self)
            elif subMenuBean.Kind == __default_menu_type_item__:
                item = wx.MenuItem(self, subMenuBean.Id, help = subMenuBean.Help)
                # item.SetBitmap(wx.Bitmap('exit.png'))
                wx.Menu.AppendItem(self, item)
                # wx.Menu.Bind(self, wx.EVT_MENU, self.onRefresh, refreshMenuItem)
            else:

               wx.Menu.Append(self, subMenuBean.Id, text= withShortCut(subMenuBean.Title, subMenuBean.Shortcut), help = subMenuBean.Help)
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




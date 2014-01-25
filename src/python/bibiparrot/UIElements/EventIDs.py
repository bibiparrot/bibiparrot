################################################################################
# Name     : EventIDs.py                                                       #
# Brief    : Since user cannot define ID by them self, I want to build         #
#         up an mapping between we self-defined ID and user ID                 #
#                                                                              #
# Url      : http://docs.wxwidgets.org/trunk/page_stdevtid.html                #
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

from ...bibiparrot.Configurations.configurations import *

from ...bibiparrot.Configurations.BibiException import *



__FIX_IDS__ = ['SEPARATOR',
    'NONE',
    'LOWEST',
    'OPEN',
    'CLOSE',
    'NEW',
    'SAVE',
    'SAVEAS',
    'REVERT',
    'EXIT',
    'UNDO',
    'REDO',
    'HELP',
    'PRINT',
    'PRINT_SETUP',
    'PAGE_SETUP',
    'PREVIEW',
    'ABOUT',
    'HELP_CONTENTS',
    'HELP_COMMANDS',
    'HELP_PROCEDURES',
    'HELP_CONTEXT',
    'HELP_INDEX',
    'HELP_SEARCH',
    'CLOSE_ALL',
    'PREFERENCES',
    'EDIT',
    'CUT',
    'COPY',
    'PASTE',
    'CLEAR',
    'FIND',
    'DUPLICATE',
    'SELECTALL',
    'DELETE',
    'REPLACE',
    'REPLACE_ALL',
    'PROPERTIES',
    'VIEW_DETAILS',
    'VIEW_LARGEICONS',
    'VIEW_SMALLICONS',
    'VIEW_LIST',
    'VIEW_SORTDATE',
    'VIEW_SORTNAME',
    'VIEW_SORTSIZE',
    'VIEW_SORTTYPE',
    'FILE',
    'FILE1',
    'FILE2',
    'FILE3',
    'FILE4',
    'FILE5',
    'FILE6',
    'FILE7',
    'FILE8',
    'FILE9',
    'OK',
    'CANCEL',
    'APPLY',
    'YES',
    'NO',
    'STATIC',
    'FORWARD',
    'BACKWARD',
    'DEFAULT',
    'MORE',
    'SETUP',
    'RESET',
    'CONTEXT_HELP',
    'YESTOALL',
    'NOTOALL',
    'ABORT',
    'RETRY',
    'IGNORE',
    'ADD',
    'REMOVE',
    'UP',
    'DOWN',
    'HOME',
    'REFRESH',
    'STOP',
    'INDEX',
    'BOLD',
    'ITALIC',
    'JUSTIFY_CENTER',
    'JUSTIFY_FILL',
    'JUSTIFY_RIGHT',
    'JUSTIFY_LEFT',
    'UNDERLINE',
    'INDENT',
    'UNINDENT',
    'ZOOM_100',
    'ZOOM_FIT',
    'ZOOM_IN',
    'ZOOM_OUT',
    'UNDELETE',
    'REVERT_TO_SAVED',
    'HIGHEST'
]


__ID_RENAME__ = {
    'OPEN':['FileOpen'],
    'CLOSE':['FileClose'],
    'JUSTIFY_LEFT':['AlignLeft'],
    'JUSTIFY_RIGHT':['AlignRight'],
    'JUSTIFY_CENTER':['AlignCenter'],
    'UNINDENT':['IndentLess'],
    'INDENT':['IndentMore']
}


FixedIDs = {}

for fixId in __FIX_IDS__:
    FixedIDs[fixId] = getattr(wx, "ID_" + fixId, wx.ID_ANY)
for key in __ID_RENAME__.keys():
    aliases = __ID_RENAME__[key]
    for alias in aliases:
        FixedIDs[alias.upper()] = FixedIDs[key]


def getIDbyLong(selfid):
    wxId, elem = eventids.get(selfid, (wx.NewId(),))
    if elem is None:
        eventids[selfid] = (wxId, elem)
    return wxId


def getIDbyElement(elem):
    selfid = getattr(elem, 'Id', -1)
    if selfid == -1:
        error = "Unknown type: %s" % type(elem)
        raise BibiException(error)
    wxId, tmp = eventids.get(selfid, (-1, None))
    if wxId == -1:
        selfname = getattr(elem, 'Name', '')
        wxId = FixedIDs.get(selfname.upper(), wx.NewId())
        eventids[selfid] = (wxId, elem)
        uielementnames[selfname] = (wxId, elem)
    return wxId


def getElementbyID(selfid):
    return eventids.get(selfid, None)

def getElementbyName(selfname):
    return uielementnames.get(selfname,None)

# @PendingDeprecationWarning
# def getID(uielem):
#     ### int ID ###
#     selfid = -1
#     if isinstance(uielem, long):
#         selfid = uielem
#     elif hasattr(uielem, "Id"):
#         selfid = uielem.Id
#     else:
#         error = "Unknown type: %s" % type(uielem)
#         raise BibiException(error)
#     ### make sure not empty ###
#     ### mapping ###
#     if eventids.has_key(selfid):
#         (wxId, uielem) = eventids[selfid]
#     elif hasattr(uielem, "Name") and FixedIDs.has_key(uielem.Name.upper()):
#         wxId = FixedIDs[uielem.Name.upper()]
#         eventids[selfid] = (wxId, uielem)
#     else:
#         wxId = wx.NewId()
#         eventids[selfid] = (wxId, uielem)
#     # print eventids
#     return wxId

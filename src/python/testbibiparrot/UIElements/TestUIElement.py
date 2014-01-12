#!/usr/bin/env python



from ...bibiparrot.UIElements.MainFrame import MainFrame
from ...bibiparrot.UIElements.UIElement import UIElement
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.Configurations.Configuration import log

import unittest
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx

class TestUIElement(unittest.TestCase):
    def setUp(self):
        self.UIElement = UIElement()
        pass

    def tearDown(self):
        pass

    def testLoad(self):
        dump = self.UIElement.dump()
        print unicode(dump).encode('utf8')
        dump = self.UIElement.loadSect("MainFrame").dump()
        print unicode(dump).encode('utf8')
        print self.UIElement.Position
        pass


if __name__ == '__main__':
    unittest.main()
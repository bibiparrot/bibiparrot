

from ...bibiparrot.Constants.constants import __required_wx_version__

import unittest
import os
import time
import logging
import wxversion
wxversion.select(__required_wx_version__)
import wx

from ...bibiparrot.Configurations.Configuration import log


class TestMediaPlayControl(unittest.TestCase):
    def setUp(self):
        import inspect
        mp3dir = os.path.join(os.path.dirname(inspect.getfile(inspect.currentframe())), 'musics4test' )
        self.mp3Path = os.path.join(mp3dir, r'Dana_Winner.mp3')
        print 'MP3 file is', self.mp3Path

        pass

    def tearDown(self):
        pass

    def testVlcCtrl(self):
        from ...bibiparrot.MediaElements.MediaPlayControl import VLCMediaPlayCtrl
        self.app = wx.App()
        frame = wx.Frame(None, size=(800,600))
        frame.Centre()
        frame.Show()
        self.ctrl = VLCMediaPlayCtrl(frame.GetHandle())
        self.ctrl.load()
        self.ctrl.open(self.mp3Path)
        self.ctrl.play()
        time.sleep(4)
        self.ctrl.stop()
        self.ctrl.start(60)
        time.sleep(4)
        self.ctrl.stop()
        self.ctrl.quit()
        # self.app.MainLoop()

    def testPygameCtrl(self):
        try:
            from ...bibiparrot.MediaElements.MediaPlayControl import PygameMediaPlayCtrl
            self.ctrl = PygameMediaPlayCtrl()

            self.ctrl = PygameMediaPlayCtrl()
            for i in range(1, 2):
                print '--------------------->', i
                self.ctrl.load()
                self.ctrl.open(self.mp3Path)
                self.ctrl.start(0)
                # self.ctrl = PygameMediaPlayCtrl()
                # while self.ctrl.isPlayed():
                #     time.sleep(0.05)
                # self.ctrl.start(60)
                time.sleep(4)
                print 'get_pos', self.ctrl.player.get_pos()
                # self.ctrl.pause()
                # time.sleep(3)
                # self.ctrl.resume()
                # time.sleep(3)
                # self.ctrl.stop()
                # time.sleep(2)
                # while self.ctrl.loaded:
                #     print '.'
                #     time.sleep(0.1)
                # time.sleep(2)
                self.ctrl.stop()
                self.ctrl.quit()

        except ImportError as err:
            print err

        # import pygame, inspect
        # ### http://www.pygame.org/docs/ref/display.html ###
        # pygame.display.init()
        # screen = pygame.display.set_mode((1024, 768))
        # background = pygame.Surface((1024, 768))
        # screen.blit(background, (0, 0))
        # pygame.display.update()
        # movpath = os.path.join(os.path.dirname(inspect.getfile(inspect.currentframe())), 'musics4test' )
        # movpath = os.path.join(movpath,'xiaoaojianghu.flv')
        # mov = pygame.movie.Movie(movpath)
        # mrect = pygame.Rect(0,0,140,113)
        # mov.set_display(screen, mrect.move(65, 150))
        # mov.set_volume(0)
        # mov.play()
        # time.sleep(6)
        # mov.stop()
        # pygame.display.quit()

        pass


if __name__ == '__main__':
    import cProfile as profile
    unittest.main()
    # profile.run('unittest.main()', sort=1)
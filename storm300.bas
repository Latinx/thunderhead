'        Verily, verily I say unto you, He that believeth on Me hath
'                 eternal life. - JOHN 6:47
'
'                        * * * * STORM * * * * *
'                              Version 4.5
'                       Lightning Simulator Program
'
'          Dan Robinson <email>>   dr@stormhighway.com
'                        St. Louis
'                Copyright Dan Robinson
'                       **** Press F5 to start ****
'                    **** Press CTRL-BREAK to Stop ****
'           **** Press SPACE to increase 'tower hit' probability ****
'                     **** Press 1 to 'freeze frame' ****
'                       **** Press 2 to 'unfreeze' ****
'     This version is designed for 300MHz or faster Pentium II computers.
'          Download executable file at:
'                  http://stormhighway.com
'
DECLARE SUB Intro ()
DECLARE SUB MyName ()
DECLARE SUB RedrawTower ()
DECLARE SUB Stormdelay ()
DECLARE SUB Quitprg ()
DECLARE SUB EndGame ()
DECLARE SUB Thunder ()
DECLARE SUB Intracloud ()
DECLARE SUB BranchOne ()
DECLARE SUB BranchTwo ()
DECLARE SUB BranchThree ()
DECLARE SUB message ()
DECLARE SUB TitleStorm ()
DECLARE SUB VersionNum ()
DECLARE SUB Verse ()
DECLARE SUB gap ()
DIM SHARED towerhit, freezemain, sheet, TME, STORMGONE, INTENSE, DUR, DEL, x, MENU, THRS, closeone, SOUNDON
DIM SHARED w, brone, brtwo, brthree, bhhone, bhhtwo, bhhthree
DIM SHARED propo, brlength, bronefactor, rc

DIM SHARED qxa(10), qxb(10), qxc(10), qxd(10), qxe(10)
DIM SHARED qxf(10), qxg(10), qxh(10), qxi(10), qxj(10)
DIM SHARED qxk(10), qxl(10), qxm(10), qxn(10), qxo(10)
DIM SHARED qxp(10), qxq(10), qxr(10), qxs(10), qxt(10)
DIM SHARED qxu(10), qxv(10), qxw(10), qxx(10)
DIM SHARED a, B, c, d, e, f, g, h, i, j, k, l, m, n, o, p
DIM SHARED ab, ac, ad, ae, af, ag, ah, ai, aj, ak
DIM SHARED al, am, an, ao, ap, aq, ar, xax, at, au
DIM SHARED av, aw, ax, ay, az, ba, bb, bc, bd, be
DIM SHARED BF, bg, bh, bi, bj, bk, bl, redun, powerflash, sparkfly, spkfl, pfl
'caps lock switch
'DEF SEG = 0
'POKE &H417, (&H40)
RANDOMIZE TIMER
MENU = 0
INTENSE = 1
11119
SCREEN 9
Intro
CLS
x = 1
l& = 1
SOUNDON = 0
MENU = 0
STORMGONE = 0
INTENSE = 1
TME = 0
THRS = 0
CLS
IF x = 4 THEN TME = 100000
IF x = 4 THEN THRS = 10000
CLS
SCREEN 9
LET h& = 0
STORMGONE = 0
DO
IF STORMGONE = 1 THEN Quitprg
IF MENU = 1 THEN GOTO 11119
LINE (0, 0)-(640, 309), 0, BF

LINE (0, 320)-(640, 400), 8, BF
RedrawTower

LINE (500, 310)-(500, 319), 8
PSET (500, 310), 4
PSET (500, 314), 4

LET h& = h& + 1
IF l& = 1 THEN GOTO 120
IF h& = 80 THEN GOSUB 64
120 LINE (0, 0)-(640, 309), 0, BF

LINE (0, 320)-(640, 400), 8, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF
RedrawTower
LET w = 0
a = INT(RND * 640) + 1
IF INKEY$ = CHR$(32) THEN a = 500
IF INKEY$ = CHR$(81) THEN Quitprg  '*****DO NOT SAVE When you exit!!!!!!
IF INKEY$ = CHR$(113) THEN Quitprg '*****DO NOT SAVE When you exit!!!!!!
freezemain = 0    '^^^^^^^^^^^^^^make sure there is not a 'q' over the 'if'!!
IF INKEY$ = CHR$(49) THEN LET freezemain = 1
closeone = INT(RND * 25) + 1
mesg = INT(RND * 300) + 1
sparkfly = 0
powerflash = 0
pfl = INT(RND * 2) + 1
IF pfl = 2 THEN powerflash = 1

spkfl = INT(RND * 2) + 1
IF spkfl = 2 THEN sparkfly = 1

IF closeone = 1 THEN sheet = 0
IF TME >= 50000 THEN sheet = 1
IF TME >= 50000 THEN closeone = 0
B = 0
brlength = 10
brlgen = 0
brone = 0
brtwo = 0
brthree = 0
bhhone = 1
bhhtwo = 1
bhhthree = 1
redun = 0
stdel = INT(RND * 9) + 1
SLEEP stdel
DO
z = INT(RND * 10) + 9
y = INT(RND * 16) + 1
brlength = brlength - .2
brlgen = brlgen + .2
LET y = y - 8
IF w = 0 THEN LET c = a + y
IF w = 0 THEN LET d = B + z
IF w = 1 THEN LET e = c + y
IF w = 1 THEN LET f = d + z
IF w = 2 THEN LET g = e + y
IF INKEY$ = CHR$(49) THEN LET freezemain = 1
IF w = 2 THEN LET h = f + z
IF w = 3 THEN LET i = g + y
IF w = 3 THEN LET j = h + z
IF w = 4 THEN LET k = i + y
IF w = 4 THEN LET l = j + z
IF w = 5 THEN LET m = k + y
IF w = 5 THEN LET n = l + z
IF w = 6 THEN LET o = m + y
IF w = 6 THEN LET p = n + z
IF w = 7 THEN LET ab = o + y
IF w = 7 THEN LET ac = p + z
IF w = 8 THEN LET ad = ab + y
IF w = 8 THEN LET ae = ac + z
IF w = 9 THEN LET af = ad + y
IF w = 9 THEN LET ag = ae + z
IF w = 10 THEN LET ah = af + y
IF w = 10 THEN LET ai = ag + z
IF w = 11 THEN LET aj = ah + y
IF w = 11 THEN LET ak = ai + z
IF w = 12 THEN LET al = aj + y
IF w = 12 THEN LET am = ak + z
IF w = 13 THEN LET an = al + y
IF w = 13 THEN LET ao = am + z
IF w = 14 THEN LET ap = an + y
IF w = 14 THEN LET aq = ao + z
IF w = 15 THEN LET ar = ap + y
IF w = 15 THEN LET xax = aq + z
IF w = 16 THEN LET at = ar + y
IF w = 16 THEN LET au = xax + z
IF w = 17 THEN LET av = at + y
IF w = 17 THEN LET aw = au + z
IF w = 18 THEN LET ax = av + y
IF w = 18 THEN LET ay = aw + z
IF w = 19 THEN LET az = ax + y
IF w = 19 THEN LET ba = ay + z
IF w = 20 THEN LET bb = az + y
IF w = 20 THEN LET bc = ba + z
IF w = 21 THEN LET bd = bb + y
IF w = 21 THEN LET be = bc + z
IF w = 22 THEN LET BF = bd + y
IF w = 22 THEN LET bg = be + z
IF w = 23 THEN LET bh = BF + y
IF w = 23 THEN LET bi = bg + z
IF w = 24 THEN LET bj = bh + y
IF w = 24 THEN LET bk = bi + z
w = w - 1
'Three seperate branch generators
IF brone = 4 THEN GOTO 236131
39131
IF brtwo = 5 THEN GOTO 236132
39132
IF brthree = 6 THEN BranchThree
IF brone = 4 THEN brone = 0
IF brtwo = 5 THEN brtwo = 0
IF brthree = 6 THEN brthree = 0
brone = brone + 1
brtwo = brtwo + 1
brthree = brthree + 1
w = w + 1

towerhit = 0
donot = 0
IF bk < 319 THEN bk = 319
IF ABS(bj - 500) < 20 AND bk < 320 THEN donot = 1
IF ABS(bj - 500) < 20 AND bk < 320 THEN bk = 310
IF ABS(bj - 500) < 20 AND bk < 320 THEN bj = 500
IF ABS(bj - 500) < 20 AND bk < 320 THEN towerhit = 1

IF inkeys$ = CHR$(49) THEN LET freezemain = 1
IF towerhit = 1 THEN LET closeone = 0
IF towerhit = 1 THEN LET powerflash = 0
LET w = w + 1
IF w = 25 THEN GOTO 2
LOOP
2 LET u = (bk / 20) - 10
IF bk > 199 THEN LET u = 40
GOSUB 55
GOSUB 39
GOSUB 51
GOSUB 93
rannum = 0
MOVE = 0
ribbon = 0
IF INKEY$ = CHR$(81) THEN Quitprg  '*****DO NOT SAVE When you exit!!!!!!
IF INKEY$ = CHR$(113) THEN Quitprg '*****DO NOT SAVE When you exit!!!!!!
w = INT(RND * 2) + 1 '^^^^^^^^^^^^^^make sure there is not a 'q' over the 'if'!!
v = INT(RND * 2) + 1
r = INT(RND * 2) + 1
q = INT(RND * 2) + 1
t = INT(RND * 2) + 1
s = INT(RND * 2) + 1



RedrawTower
LINE (a, B)-(c, d)   'S=1 IS nonbranched

3      'FIRST RETURN STROKE
'branches
LINE (qxa(1), qxb(1))-(qxc(1), qxd(1))
LINE (qxc(1), qxd(1))-(qxe(1), qxf(1))
LINE (qxe(1), qxf(1))-(qxg(1), qxh(1))
LINE (qxg(1), qxh(1))-(qxi(1), qxj(1))
LINE (qxi(1), qxj(1))-(qxk(1), qxl(1))
LINE (qxk(1), qxl(1))-(qxm(1), qxn(1))
LINE (qxm(1), qxn(1))-(qxo(1), qxp(1))
LINE (qxo(1), qxp(1))-(qxq(1), qxr(1))
LINE (qxq(1), qxr(1))-(qxs(1), qxt(1))
LINE (qxs(1), qxt(1))-(qxu(1), qxv(1))
LINE (qxu(1), qxv(1))-(qxw(1), qxx(1))
LINE (qxas(1), qxbs(1))-(qxcs(1), qxds(1))
LINE (qxcs(1), qxds(1))-(qxes(1), qxfs(1))
LINE (qxes(1), qxfs(1))-(qxgs(1), qxhs(1))
LINE (qxast(1), qxbst(1))-(qxcst(1), qxdst(1))
LINE (qxcst(1), qxdst(1))-(qxest(1), qxfst(1))
LINE (qxest(1), qxfst(1))-(qxgst(1), qxhst(1))

LINE (qxa(2), qxb(2))-(qxc(2), qxd(2))
LINE (qxc(2), qxd(2))-(qxe(2), qxf(2))
LINE (qxe(2), qxf(2))-(qxg(2), qxh(2))
LINE (qxg(2), qxh(2))-(qxi(2), qxj(2))
LINE (qxi(2), qxj(2))-(qxk(2), qxl(2))
LINE (qxk(2), qxl(2))-(qxm(2), qxn(2))
LINE (qxm(2), qxn(2))-(qxo(2), qxp(2))
LINE (qxo(2), qxp(2))-(qxq(2), qxr(2))
LINE (qxq(2), qxr(2))-(qxs(2), qxt(2))
LINE (qxs(2), qxt(2))-(qxu(2), qxv(2))
LINE (qxu(2), qxv(2))-(qxw(2), qxx(2))
LINE (qxas(2), qxbs(2))-(qxcs(2), qxds(2))
LINE (qxcs(2), qxds(2))-(qxes(2), qxfs(2))
LINE (qxes(2), qxfs(2))-(qxgs(2), qxhs(2))
LINE (qxast(2), qxbst(2))-(qxcst(2), qxdst(2))
LINE (qxcst(2), qxdst(2))-(qxest(2), qxfst(2))
LINE (qxest(2), qxfst(2))-(qxgst(2), qxhst(2))

LINE (qxa(3), qxb(3))-(qxc(3), qxd(3))
LINE (qxc(3), qxd(3))-(qxe(3), qxf(3))
LINE (qxe(3), qxf(3))-(qxg(3), qxh(3))
LINE (qxg(3), qxh(3))-(qxi(3), qxj(3))
LINE (qxi(3), qxj(3))-(qxk(3), qxl(3))
LINE (qxk(3), qxl(3))-(qxm(3), qxn(3))
LINE (qxm(3), qxn(3))-(qxo(3), qxp(3))
LINE (qxo(3), qxp(3))-(qxq(3), qxr(3))
LINE (qxq(3), qxr(3))-(qxs(3), qxt(3))
LINE (qxs(3), qxt(3))-(qxu(3), qxv(3))
LINE (qxu(3), qxv(3))-(qxw(3), qxx(3))
LINE (qxas(3), qxbs(3))-(qxcs(3), qxds(3))
LINE (qxcs(3), qxds(3))-(qxes(3), qxfs(3))
LINE (qxes(3), qxfs(3))-(qxgs(3), qxhs(3))
LINE (qxast(3), qxbst(3))-(qxcst(3), qxdst(3))
LINE (qxcst(3), qxdst(3))-(qxest(3), qxfst(3))
LINE (qxest(3), qxfst(3))-(qxgst(3), qxhst(3))

LINE (qxa(4), qxb(4))-(qxc(4), qxd(4))
LINE (qxc(4), qxd(4))-(qxe(4), qxf(4))
LINE (qxe(4), qxf(4))-(qxg(4), qxh(4))
LINE (qxg(4), qxh(4))-(qxi(4), qxj(4))
LINE (qxi(4), qxj(4))-(qxk(4), qxl(4))
LINE (qxk(4), qxl(4))-(qxm(4), qxn(4))
LINE (qxm(4), qxn(4))-(qxo(4), qxp(4))
LINE (qxo(4), qxp(4))-(qxq(4), qxr(4))
LINE (qxq(4), qxr(4))-(qxs(4), qxt(4))
LINE (qxs(4), qxt(4))-(qxu(4), qxv(4))
LINE (qxu(4), qxv(4))-(qxw(4), qxx(4))
LINE (qxas(4), qxbs(4))-(qxcs(4), qxds(4))
LINE (qxcs(4), qxds(4))-(qxes(4), qxfs(4))
LINE (qxes(4), qxfs(4))-(qxgs(4), qxhs(4))
LINE (qxast(4), qxbst(4))-(qxcst(4), qxdst(4))
LINE (qxcst(4), qxdst(4))-(qxest(4), qxfst(4))
LINE (qxest(4), qxfst(4))-(qxgst(4), qxhst(4))



LINE (qqxa(1), qqxb(1))-(qqxc(1), qqxd(1))
LINE (qqxc(1), qqxd(1))-(qqxe(1), qqxf(1))
LINE (qqxe(1), qqxf(1))-(qqxg(1), qqxh(1))
LINE (qqxg(1), qqxh(1))-(qqxi(1), qqxj(1))
LINE (qqxi(1), qqxj(1))-(qqxk(1), qqxl(1))
LINE (qqxk(1), qqxl(1))-(qqxm(1), qqxn(1))
LINE (qqxm(1), qqxn(1))-(qqxo(1), qqxp(1))
LINE (qqxo(1), qqxp(1))-(qqxq(1), qqxr(1))
LINE (qqxq(1), qqxr(1))-(qqxs(1), qqxt(1))
LINE (qqxs(1), qqxt(1))-(qqxu(1), qqxv(1))
LINE (qqxu(1), qqxv(1))-(qqxw(1), qqxx(1))
LINE (qqxas(1), qqxbs(1))-(qqxcs(1), qqxds(1))
LINE (qqxcs(1), qqxds(1))-(qqxes(1), qqxfs(1))
LINE (qqxes(1), qqxfs(1))-(qqxgs(1), qqxhs(1))
LINE (qqxast(1), qqxbst(1))-(qqxcst(1), qqxdst(1))
LINE (qqxcst(1), qqxdst(1))-(qqxest(1), qqxfst(1))
LINE (qqxest(1), qqxfst(1))-(qqxgst(1), qqxhst(1))

LINE (qqxa(2), qqxb(2))-(qqxc(2), qqxd(2))
LINE (qqxc(2), qqxd(2))-(qqxe(2), qqxf(2))
LINE (qqxe(2), qqxf(2))-(qqxg(2), qqxh(2))
LINE (qqxg(2), qqxh(2))-(qqxi(2), qqxj(2))
LINE (qqxi(2), qqxj(2))-(qqxk(2), qqxl(2))
LINE (qqxk(2), qqxl(2))-(qqxm(2), qqxn(2))
LINE (qqxm(2), qqxn(2))-(qqxo(2), qqxp(2))
LINE (qqxo(2), qqxp(2))-(qqxq(2), qqxr(2))
LINE (qqxq(2), qqxr(2))-(qqxs(2), qqxt(2))
LINE (qqxs(2), qqxt(2))-(qqxu(2), qqxv(2))
LINE (qqxu(2), qqxv(2))-(qqxw(2), qqxx(2))
LINE (qqxas(2), qqxbs(2))-(qqxcs(2), qqxds(2))
LINE (qqxcs(2), qqxds(2))-(qqxes(2), qqxfs(2))
LINE (qqxes(2), qqxfs(2))-(qqxgs(2), qqxhs(2))
LINE (qqxast(2), qqxbst(2))-(qqxcst(2), qqxdst(2))
LINE (qqxcst(2), qqxdst(2))-(qqxest(2), qqxfst(2))
LINE (qqxest(2), qqxfst(2))-(qqxgst(2), qqxhst(2))

LINE (qqxa(3), qqxb(3))-(qqxc(3), qqxd(3))
LINE (qqxc(3), qqxd(3))-(qqxe(3), qqxf(3))
LINE (qqxe(3), qqxf(3))-(qqxg(3), qqxh(3))
LINE (qqxg(3), qqxh(3))-(qqxi(3), qqxj(3))
LINE (qqxi(3), qqxj(3))-(qqxk(3), qqxl(3))
LINE (qqxk(3), qqxl(3))-(qqxm(3), qqxn(3))
LINE (qqxm(3), qqxn(3))-(qqxo(3), qqxp(3))
LINE (qqxo(3), qqxp(3))-(qqxq(3), qqxr(3))
LINE (qqxq(3), qqxr(3))-(qqxs(3), qqxt(3))
LINE (qqxs(3), qqxt(3))-(qqxu(3), qqxv(3))
LINE (qqxu(3), qqxv(3))-(qqxw(3), qqxx(3))
LINE (qqxas(3), qqxbs(3))-(qqxcs(3), qqxds(3))
LINE (qqxcs(3), qqxds(3))-(qqxes(3), qqxfs(3))
LINE (qqxes(3), qqxfs(3))-(qqxgs(3), qqxhs(3))
LINE (qqxast(3), qqxbst(3))-(qqxcst(3), qqxdst(3))
LINE (qqxcst(3), qqxdst(3))-(qqxest(3), qqxfst(3))
LINE (qqxest(3), qqxfst(3))-(qqxgst(3), qqxhst(3))

LINE (qqxa(4), qqxb(4))-(qqxc(4), qqxd(4))
LINE (qqxc(4), qqxd(4))-(qqxe(4), qqxf(4))
LINE (qqxe(4), qqxf(4))-(qqxg(4), qqxh(4))
LINE (qqxg(4), qqxh(4))-(qqxi(4), qqxj(4))
LINE (qqxi(4), qqxj(4))-(qqxk(4), qqxl(4))
LINE (qqxk(4), qqxl(4))-(qqxm(4), qqxn(4))
LINE (qqxm(4), qqxn(4))-(qqxo(4), qqxp(4))
LINE (qqxo(4), qqxp(4))-(qqxq(4), qqxr(4))
LINE (qqxq(4), qqxr(4))-(qqxs(4), qqxt(4))
LINE (qqxs(4), qqxt(4))-(qqxu(4), qqxv(4))
LINE (qqxu(4), qqxv(4))-(qqxw(4), qqxx(4))
LINE (qqxas(4), qqxbs(4))-(qqxcs(4), qqxds(4))
LINE (qqxcs(4), qqxds(4))-(qqxes(4), qqxfs(4))
LINE (qqxes(4), qqxfs(4))-(qqxgs(4), qqxhs(4))
LINE (qqxast(4), qqxbst(4))-(qqxcst(4), qqxdst(4))
LINE (qqxcst(4), qqxdst(4))-(qqxest(4), qqxfst(4))
LINE (qqxest(4), qqxfst(4))-(qqxgst(4), qqxhst(4))


'main channel
LINE (a, B)-(c, d)
LINE (a + 1, B)-(c + 1, d)
LINE (a + 2, B)-(c + 2, d)
LINE (a - 1, B)-(c - 1, d)     '
LINE (c, d)-(e, f)
LINE (c + 1, d)-(e + 1, f)
LINE (c + 2, d)-(e + 2, f)
LINE (c - 1, d)-(e - 1, f)     '
LINE (e, f)-(g, h)
LINE (e + 1, f)-(g + 1, h)
LINE (e + 2, f)-(g + 2, h)
LINE (e - 1, f)-(g - 1, h)     '
LINE (g, h)-(i, j)
LINE (g + 1, h)-(i + 1, j)
LINE (g + 2, h)-(i + 2, j)
LINE (g - 1, h)-(i - 1, j)         '
LINE (i, j)-(k, l)
7 LINE (i + 1, j)-(k + 1, l)
LINE (i + 2, j)-(k + 2, l)
LINE (i - 1, j)-(k - 1, l)          '
LINE (k, l)-(m, n)
LINE (k + 1, l)-(m + 1, n)
LINE (k + 2, l)-(m + 2, n)
LINE (k - 1, l)-(m - 1, n)           '
LINE (m, n)-(o, p)
LINE (m + 1, n)-(o + 1, p)
LINE (m + 2, n)-(o + 2, p)
LINE (m - 1, n)-(o - 1, p)          '
LINE (o, p)-(ab, ac)
LINE (o + 1, p)-(ab + 1, ac)
LINE (o + 2, p)-(ab + 2, ac)
LINE (o - 1, p)-(ab - 1, ac)
LINE (ab, ac)-(ad, ae)
LINE (ab + 1, ac)-(ad + 1, ae)
LINE (ab + 2, ac)-(ad + 2, ae)
LINE (ab - 1, ac)-(ad - 1, ae)
LINE (ad, ae)-(af, ag)
LINE (ad + 1, ae)-(af + 1, ag)
LINE (ad + 2, ae)-(af + 2, ag)
LINE (ad - 1, ae)-(af - 1, ag)
8 LINE (af, ag)-(ah, ai)
LINE (af + 1, ag)-(ah + 1, ai)
LINE (af + 2, ag)-(ah + 2, ai)
LINE (af - 1, ag)-(ah - 1, ai)
LINE (ah, ai)-(aj, ak)
LINE (ah + 1, ai)-(aj + 1, ak)
LINE (ah + 2, ai)-(aj + 2, ak)
LINE (ah - 1, ai)-(aj - 1, ak)
LINE (aj, ak)-(al, am)
LINE (aj + 1, ak)-(al + 1, am)
LINE (aj + 2, ak)-(al + 2, am)
LINE (aj - 1, ak)-(al - 1, am)
4 LINE (al, am)-(an, ao)
LINE (al + 1, am)-(an + 1, ao)
LINE (al + 2, am)-(an + 2, ao)
LINE (al - 1, am)-(an - 1, ao)
LINE (an, ao)-(ap, aq)
LINE (an + 1, ao)-(ap + 1, aq)
LINE (an + 2, ao)-(ap + 2, aq)
LINE (an - 1, ao)-(ap - 1, aq)
LINE (ap, aq)-(ar, xax)
LINE (ap + 1, aq)-(ar + 1, xax)
LINE (ap + 2, aq)-(ar + 2, xax)
LINE (ap - 1, aq)-(ar - 1, xax)
LINE (ar, xax)-(at, au)
LINE (ar + 1, xax)-(at + 1, au)
LINE (ar + 2, xax)-(at + 2, au)
LINE (ar - 1, xax)-(at - 1, au)
6 LINE (at, au)-(av, aw)
LINE (at + 1, au)-(av + 1, aw)
LINE (at + 2, au)-(av + 2, aw)
LINE (at - 1, au)-(av - 1, aw)
LINE (av, aw)-(ax, ay)
LINE (av + 1, aw)-(ax + 1, ay)
LINE (av + 2, aw)-(ax + 2, ay)
LINE (av - 1, aw)-(ax - 1, ay)
LINE (ax, ay)-(az, ba)
LINE (ax + 1, ay)-(az + 1, ba)
LINE (ax + 2, ay)-(az + 2, ba)
LINE (ax - 1, ay)-(az - 1, ba)
LINE (az, ba)-(bb, bc)
LINE (az + 1, ba)-(bb + 1, bc)
LINE (az + 2, ba)-(bb + 2, bc)
LINE (az - 1, ba)-(bb - 1, bc)
LINE (bb, bc)-(bd, be)
LINE (bb + 1, bc)-(bd + 1, be)
LINE (bb + 2, bc)-(bd + 2, be)
LINE (bb - 1, bc)-(bd - 1, be)
5 LINE (bd, be)-(BF, bg)
LINE (bd + 1, be)-(BF + 1, bg)
LINE (bd + 2, be)-(BF + 2, bg)
LINE (bd - 1, be)-(BF - 1, bg)
LINE (BF, bg)-(bh, bi)
LINE (BF + 1, bg)-(bh + 1, bi)
LINE (BF + 2, bg)-(bh + 2, bi)
LINE (BF - 1, bg)-(bh - 1, bi)
LINE (bh, bi)-(bj, bk)
LINE (bh + 1, bi)-(bj + 1, bk)
LINE (bh + 2, bi)-(bj + 2, bk)
LINE (bh - 1, bi)-(bj - 1, bk)
LINE (bj, bk + 1)-(bj + 1, bk + 1)
IF closeone = 1 THEN LINE (0, 0)-(640, 400), 7, BF
IF closeone = 1 THEN Thunder
IF freezemain = 1 THEN GOSUB 10191
33221
GOSUB 77

LINE (0, 0)-(640, 309), 0, BF

LINE (0, 320)-(640, 400), 8, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF
RedrawTower
GOSUB 45
GOSUB 47
IF closeone = 1 THEN GOTO 54301
Thunder
54301
towerhit = 0
RedrawTower
intra = INT(RND * 10) + 1
IF intra = 1 THEN Intracloud
RedrawTower
IF powerflash = 1 THEN GOSUB 15
IF powerflash = 1 THEN sparkfly = 0
IF sparkfly = 1 THEN GOSUB 85
LET t = 0
z = INT(RND * 10) + 1
q = INT(RND * 4) + 1
IF x = 3 THEN LET q = q + 10
IF x = 1 THEN GOTO 87
IF x = 4 THEN Stormdelay
IF x <> 4 THEN SLEEP q

87 LOOP
45 MOVE = 0
rib = 0
ribbon = INT(RND * 6) + 1
rannum = INT(RND * 2) + 1
IF rannum = 1 THEN mover = .25 ELSE mover = -.25
IF ribbon = 1 THEN MOVE = mover
y = INT(RND * 15) + 1
z = INT(RND * 10) + 1
IF z > 7 THEN LET y = 1
IF z < 4 THEN RETURN
s = 0

DO
LINE (0, 0)-(640, 309), 0, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (501, 310)-(640, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (0, 320)-(640, 400), 8, BF


RedrawTower
LINE (a + 1 + rib, B)-(c + 1 + rib, d)
LINE (c + 1 + rib, d)-(e + 1 + rib, f)
LINE (e + 1 + rib, f)-(g + 1 + rib, h)
LINE (g + 1 + rib, h)-(i + 1 + rib, j)
LINE (i + 1 + rib, j)-(k + 1 + rib, l)
LINE (k + 1 + rib, l)-(m + 1 + rib, n)
LINE (m + 1 + rib, n)-(o + 1 + rib, p)
LINE (o + 1 + rib, p)-(ab + 1 + rib, ac)
LINE (ab + 1 + rib, ac)-(ad + 1 + rib, ae)
LINE (ad + 1 + rib, ae)-(af + 1 + rib, ag)
LINE (af + 1 + rib, ag)-(ah + 1 + rib, ai)
LINE (ah + 1 + rib, ai)-(aj + 1 + rib, ak)
LINE (aj + 1 + rib, ak)-(al + 1 + rib, am)
LINE (al + 1 + rib, am)-(an + 1 + rib, ao)
LINE (an + 1 + rib, ao)-(ap + 1 + rib, aq)
LINE (ap + 1 + rib, aq)-(ar + 1 + rib, xax)
LINE (ar + 1 + rib, xax)-(at + 1 + rib, au)
LINE (at + 1 + rib, au)-(av + 1 + rib, aw)
LINE (av + 1 + rib, aw)-(ax + 1 + rib, ay)
LINE (ax + 1 + rib, ay)-(az + 1 + rib, ba)
LINE (az + 1 + rib, ba)-(bb + 1 + rib, bc)
LINE (bb + 1 + rib, bc)-(bd + 1 + rib, be)
LINE (bd + 1 + rib, be)-(BF + 1 + rib, bg)
LINE (BF + 1 + rib, bg)-(bh + 1 + rib, bi)
LINE (bh + 1 + rib, bi)-(bj + 1, bk)
LET rib = rib + MOVE
IF closeone = 1 THEN LINE (0, 0)-(640, 400), 7, BF
33225
GOSUB 36
LINE (0, 0)-(640, 309), 0, BF

LINE (0, 320)-(640, 400), 8, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF
RedrawTower
GOSUB 77
LET s = s + 1
IF s = y THEN RETURN
LOOP
s = 0
47 z = INT(RND * 10) + 1
IF z > 4 THEN RETURN
w = INT(RND * 3) + 1
LET t = 0
DO
LINE (0, 0)-(640, 309), 0, BF

LINE (0, 320)-(640, 400), 8, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF



RedrawTower
LINE (a + 1 + rib, B)-(c + 1 + rib, d)
LINE (c + 1 + rib, d)-(e + 1 + rib, f)
LINE (e + 1 + rib, f)-(g + 1 + rib, h)
LINE (g + 1 + rib, h)-(i + 1 + rib, j)
LINE (i + 1 + rib, j)-(k + 1 + rib, l)
LINE (k + 1 + rib, l)-(m + 1 + rib, n)
LINE (m + 1 + rib, n)-(o + 1 + rib, p)
LINE (o + 1 + rib, p)-(ab + 1 + rib, ac)
LINE (ab + 1 + rib, ac)-(ad + 1 + rib, ae)
LINE (ad + 1 + rib, ae)-(af + 1 + rib, ag)
LINE (af + 1 + rib, ag)-(ah + 1 + rib, ai)
LINE (ah + 1 + rib, ai)-(aj + 1 + rib, ak)
LINE (aj + 1 + rib, ak)-(al + 1 + rib, am)
LINE (al + 1 + rib, am)-(an + 1 + rib, ao)
LINE (an + 1 + rib, ao)-(ap + 1 + rib, aq)
LINE (ap + 1 + rib, aq)-(ar + 1 + rib, xax)
LINE (ar + 1 + rib, xax)-(at + 1 + rib, au)
LINE (at + 1 + rib, au)-(av + 1 + rib, aw)
LINE (av + 1 + rib, aw)-(ax + 1 + rib, ay)
LINE (ax + 1 + rib, ay)-(az + 1 + rib, ba)
LINE (az + 1 + rib, ba)-(bb + 1 + rib, bc)
LINE (bb + 1 + rib, bc)-(bd + 1 + rib, be)
LINE (bd + 1 + rib, be)-(BF + 1 + rib, bg)
LINE (BF + 1 + rib, bg)-(bh + 1 + rib, bi)
LINE (bh + 1 + rib, bi)-(bj + 1, bk)
LET rib = rib + MOVE
IF closeone = 1 THEN LINE (0, 0)-(640, 400), 7, BF
33229
GOSUB 36
LINE (0, 0)-(640, 309), 0, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF


LINE (0, 320)-(640, 400), 8, BF
RedrawTower
GOSUB 99
LET t = t + 1
IF t = w THEN RETURN

LOOP
77 LET u = 0
DO
LET u = u + 1
IF u = 2000 THEN RETURN
LOOP
99 LET u = 0
DO
LET u = u + 1

IF u = 4800 THEN RETURN
LOOP
36 LET u = 0
DO
LET u = u + 1
IF u = 700 THEN RETURN
LOOP
55 r = INT(RND * 10) + 1
z = INT(RND * 10) + 3
y = INT(RND * 16) + 1
LET y = y - 12
IF INKEY$ = CHR$(81) THEN Quitprg  '*****DO NOT SAVE When you exit!!!!!!
IF INKEY$ = CHR$(113) THEN Quitprg '*****DO NOT SAVE When you exit!!!!!!
RETURN   '^^^^^^^^^^^^^^make sure there is not a 'q' over the 'if'!!
66 LET u = 0
DO
LET u = u + 1
IF u = 180 THEN RETURN
LOOP
49 v = INT(RND * 30) + 1
SLEEP v
PLAY "mbt160l19o0aaacbcddaaddacl32bddal19edcbadabdca"
RETURN
85 IF bj > 199 THEN RETURN

PSET (bj, bk), 4
GOSUB 77
PSET (bj, bk), 4
PSET (bj + 1, bk - 1), 14
PSET (bj - 1, bk - 1), 14
GOSUB 77
PSET (bj, bk), 7
PSET (bj + 1, bk - 1), 8
PSET (bj - 1, bk - 1), 8
PSET (bj - 2, bk - 2), 4
PSET (bj + 2, bk - 2), 4
GOSUB 77
PSET (bj, bk), 4
PSET (bj - 2, bk - 2), 8
PSET (bj + 2, bk - 2), 8
PSET (bj + 3, bk - 1), 14
PSET (bj - 3, bk - 1), 14
PSET (bj - 3, bk - 3), 14
PSET (bj + 3, bk - 3), 14
GOSUB 77
PSET (bj, bk), 4
PSET (bj + 3, bk - 1), 8
PSET (bj - 3, bk - 1), 8
PSET (bj - 3, bk - 3), 8
PSET (bj + 3, bk - 3), 8
PSET (bj + 4, bk), 4
PSET (bj - 4, bk), 4
PSET (bj + 4, bk - 4), 14
PSET (bj + 4, bk - 4), 14
GOSUB 77
PSET (bj, bk), 4
PSET (bj, bk - 1), 4
PSET (bj + 4, bk), 8
PSET (bj - 4, bk), 8
PSET (bj - 4, bk - 4), 8
PSET (bj + 4, bk - 4), 8
PSET (bj - 5, bk - 3), 4
PSET (bj + 5, bk - 3), 4
GOSUB 77
PSET (bj - 5, bk - 3), 8
PSET (bj + 5, bk - 3), 8
PSET (bj - 6, bk - 2), 14
PSET (bj + 6, bk - 2), 14
GOSUB 77
PSET (bj + 6, bk - 2), 8
PSET (bj - 6, bk - 2), 8
PSET (bj - 7, bk - 1), 4
PSET (bj + 7, bk - 1), 4
GOSUB 77
PSET (bj + 7, bk - 1), 8
PSET (bj - 7, bk - 1), 8
PSET (bj - 8, bk), 4
PSET (bj + 8, bk), 4
GOSUB 66
PSET (bj - 8, bk), 8
PSET (bj + 8, bk), 8
GOSUB 66
PSET (bj, bk), 8
PSET (bj, bk - 1), 8
LINE (0, 320)-(640, 400), 8, BF
RETURN
15 IF bj > 199 THEN RETURN

GOSUB 36
LET a = 0
DO
IF a = 0 THEN LET c = 1
IF a = 1 THEN LET c = 3
IF a = 2 THEN LET c = 1
IF a = 3 THEN LET c = 1
IF a = 4 THEN LET c = 3
IF a = 5 THEN LET c = 2
IF a = 6 THEN LET c = 1
IF a = 7 THEN LET c = 3
IF a = 8 THEN LET c = 1
IF a = 9 THEN LET c = 8
z# = INT(RND * 4) + 1
v# = INT(RND * 4) + 1
y& = INT(RND * 2) + 1
IF y& = 1 THEN LET z# = -z#
PSET (bj, bk), c
PSET (bj + 1, bk), c
PSET (bj + 1, bk - 1), c
PSET (bj - 1, bk - 1), c
PSET (bj - 1, bk), c
GOSUB 36
PSET (bj, bk - 1), c
GOSUB 36
PSET (bj + z#, bk - v#), 2
PSET (bj + 2, bk - 1), c
PSET (bj - 2, bk - 1), c
GOSUB 36
PSET (bj, bk - 2), c
PSET (bj - 1, bk - 2), c
PSET (bj + 1, bk - 2), c
PSET (bj + z#, bk - v#), 8
GOSUB 36
LET a = a + 1
IF a = 11 THEN GOTO 41
LOOP
41 PSET (bj, bk), 0
LINE (0, 320)-(640, 400), 8, BF
RETURN
39 LET w = 0
DO
z = INT(RND * u) + (u / 2)
y = INT(RND * (u * 2)) + u
r = INT(RND * 2) + 1
IF r = 1 THEN LET z = -z
IF w = 0 THEN LET a% = a + z
IF w = 0 THEN LET B% = B + y
IF w = 1 THEN LET c% = c + z
IF w = 1 THEN LET d% = d + y
IF w = 2 THEN LET e% = e + z
IF w = 2 THEN LET f% = f + y
IF w = 3 THEN LET g% = g + z
IF w = 3 THEN LET h% = h + y
IF w = 4 THEN LET i% = i + z
IF w = 4 THEN LET j% = j + y
IF w = 5 THEN LET k% = k + z
IF w = 5 THEN LET l% = l + y
IF w = 6 THEN LET m% = m + z
IF w = 6 THEN LET n% = n + y
LET w = w + 1
IF w = 7 THEN RETURN
LOOP
51 LET w = 0
DO
y = INT(RND * 10) + 1
z = INT(RND * u) + (u / 2)
r = INT(RND * 2) + 1
IF r = 1 THEN LET y = -y
IF w = 0 THEN LET o% = a% + y
IF w = 0 THEN LET p% = B% + z
IF w = 1 THEN LET q% = c% + y
IF w = 1 THEN LET r% = d% + z
IF w = 2 THEN LET s% = e% + y
IF w = 2 THEN LET t% = f% + z
IF w = 3 THEN LET u% = g% + y
IF w = 3 THEN LET v% = h% + z
IF w = 4 THEN LET w% = i% + y
IF w = 4 THEN LET x% = j% + z
IF w = 5 THEN LET y% = k% + y
IF w = 5 THEN LET z% = l% + z
LET w = w + 1
IF w = 6 THEN RETURN
LOOP
93 LET w = 0
DO
y = INT(RND * u) + u / 2
z = INT(RND * 10) + 1
r = INT(RND * 2) + 1
IF r = 1 THEN LET z = -z
IF w = 0 THEN LET a# = a% + z
IF w = 0 THEN LET B# = B% + y
IF w = 1 THEN LET c# = c% + z
IF w = 1 THEN LET d# = d% + y
IF w = 2 THEN LET e# = e% + z
IF w = 2 THEN LET f# = f% + y
IF w = 3 THEN LET g# = g% + z
IF w = 3 THEN LET h# = h% + y
IF w = 4 THEN LET i# = i% + z
IF w = 4 THEN LET j# = j% + y
IF w = 5 THEN LET k# = k% + z
IF w = 5 THEN LET l# = l% + y
LET w = w + 1
IF w = 6 THEN RETURN
LOOP
64 LINE (160, 199)-(160, 100)
LINE (161, 199)-(161, 100)
LINE (162, 100)-(200, 115), 1
LINE (166, 120)-(170, 120), 2
LINE (168, 120)-(168, 110), 2
LINE (168, 110)-(166, 112), 2
CIRCLE (175, 112), 3, 2
CIRCLE (175, 118), 3, 2
LINE (174, 115)-(176, 115), 2
LINE (200, 115)-(162, 130), 1
PRINT "What a Great Time for Golf!"
LET h& = 0
SLEEP 4
RETURN
10191
DO
IF INKEY$ = CHR$(50) THEN GOTO 30102
LOOP
30102
RETURN
236131                                     'BRANCH ONE SUB
'set starting point
IF w = 0 THEN qxa(bhhone) = a
IF w = 0 THEN qxb(bhhone) = B
IF w = 1 THEN qxa(bhhone) = c
IF w = 1 THEN qxb(bhhone) = d
IF w = 2 THEN qxa(bhhone) = e
IF w = 2 THEN qxb(bhhone) = f
IF w = 3 THEN qxa(bhhone) = g
IF w = 3 THEN qxb(bhhone) = h
IF w = 4 THEN qxa(bhhone) = i
IF w = 4 THEN qxb(bhhone) = j
IF w = 5 THEN qxa(bhhone) = k
IF w = 5 THEN qxb(bhhone) = l
IF w = 6 THEN qxa(bhhone) = m
IF w = 6 THEN qxb(bhhone) = n
IF w = 7 THEN qxa(bhhone) = o
IF w = 7 THEN qxb(bhhone) = p
IF w = 8 THEN qxa(bhhone) = ab
IF w = 8 THEN qxb(bhhone) = ac
IF w = 9 THEN qxa(bhhone) = ad
IF w = 9 THEN qxb(bhhone) = ae
IF w = 10 THEN qxa(bhhone) = af
IF w = 10 THEN qxb(bhhone) = ag
IF w = 11 THEN qxa(bhhone) = ah
IF w = 11 THEN qxb(bhhone) = ai
IF w = 12 THEN qxa(bhhone) = aj
IF w = 12 THEN qxb(bhhone) = ak
IF w = 13 THEN qxa(bhhone) = al
IF w = 13 THEN qxb(bhhone) = am
IF w = 14 THEN qxa(bhhone) = an
IF w = 14 THEN qxb(bhhone) = ao
IF w = 15 THEN qxa(bhhone) = ap
IF w = 15 THEN qxb(bhhone) = aq
IF w = 16 THEN qxa(bhhone) = ar
IF w = 16 THEN qxb(bhhone) = xax
IF w = 17 THEN qxa(bhhone) = at
IF w = 17 THEN qxb(bhhone) = au
IF w = 18 THEN qxa(bhhone) = av
IF w = 18 THEN qxb(bhhone) = aw
IF w = 19 THEN qxa(bhhone) = ax
IF w = 19 THEN qxb(bhhone) = ay
IF w = 20 THEN qxa(bhhone) = az
IF w = 20 THEN qxb(bhhone) = ba
IF w = 21 THEN qxa(bhhone) = bb
IF w = 21 THEN qxb(bhhone) = bc
IF w = 22 THEN qxa(bhhone) = bd
IF w = 22 THEN qxb(bhhone) = be
IF w = 23 THEN qxa(bhhone) = BF
IF w = 23 THEN qxb(bhhone) = bg
IF w = 24 THEN qxa(bhhone) = bh
IF w = 24 THEN qxb(bhhone) = bi
                   
'determine general propogation direction
bdir = INT(RND * 2) + 1
redun = redun + bronefactor
IF ABS(redun) > 2 THEN bronefactor = -bronefactor
IF ABS(redun) > 2 THEN redun = 0
bronefactor = -1
IF bdir = 1 THEN bronefactor = 1
propo = (brlength * bronefactor)
bronesubone = 0
bronesubtwo = 0
brw = 0
DO
xx = INT(RND * (5 - brlgen)) + (5 - brlgen)
yy = INT(RND * (14 - brlgen)) + (3 - brlgen)
xx = xx * bronefactor
IF brw = 0 THEN qxc(bhhone) = qxa(bhhone) + xx + propo
IF brw = 0 THEN qxd(bhhone) = qxb(bhhone) + yy
IF brw = 1 THEN qxe(bhhone) = qxc(bhhone) + xx + propo
IF brw = 1 THEN qxf(bhhone) = qxd(bhhone) + yy
IF brw = 2 THEN qxg(bhhone) = qxe(bhhone) + xx + propo
IF brw = 2 THEN qxh(bhhone) = qxf(bhhone) + yy
IF brw = 3 THEN qxi(bhhone) = qxg(bhhone) + xx + propo
IF brw = 3 THEN qxj(bhhone) = qxh(bhhone) + yy
IF brw = 4 THEN qxk(bhhone) = qxi(bhhone) + xx + propo
IF brw = 4 THEN qxl(bhhone) = qxj(bhhone) + yy
IF brw = 5 THEN qxm(bhhone) = qxk(bhhone) + xx + propo
IF brw = 5 THEN qxn(bhhone) = qxl(bhhone) + yy
IF brw = 6 THEN qxo(bhhone) = qxm(bhhone) + xx + propo
IF brw = 6 THEN qxp(bhhone) = qxn(bhhone) + yy
IF brw = 7 THEN qxq(bhhone) = qxo(bhhone) + xx + propo
IF brw = 7 THEN qxr(bhhone) = qxp(bhhone) + yy
IF brw = 8 THEN qxs(bhhone) = qxq(bhhone) + xx + propo
IF brw = 8 THEN qxt(bhhone) = qxr(bhhone) + yy
IF brw = 9 THEN qxu(bhhone) = qxs(bhhone) + xx + propo
IF brw = 9 THEN qxv(bhhone) = qxt(bhhone) + yy
IF brw = 10 THEN qxw(bhhone) = qxu(bhhone) + xx + propo
IF brw = 10 THEN qxx(bhhone) = qxv(bhhone) + yy
IF bronesubone = 3 THEN GOTO 2361311
391311
IF bronesubtwo = 5 THEN GOTO 2361312
391312
bronesubone = bronesubone + 1
bronesubtwo = bronesubtwo + 1




brw = brw + 1
IF brw = 11 THEN GOTO 1263
LOOP
1263
bhhone = bhhone + 1
GOTO 39131
2361311 'first branch sub branch
IF brw = 0 THEN qxas(bhhone) = qxc(bhhone)
IF brw = 0 THEN qxbs(bhhone) = qxd(bhhone)
IF brw = 1 THEN qxas(bhhone) = qxe(bhhone)
IF brw = 1 THEN qxbs(bhhone) = qxf(bhhone)
IF brw = 2 THEN qxas(bhhone) = qxg(bhhone)
IF brw = 2 THEN qxbs(bhhone) = qxh(bhhone)
IF brw = 3 THEN qxas(bhhone) = qxi(bhhone)
IF brw = 3 THEN qxbs(bhhone) = qxj(bhhone)
IF brw = 4 THEN qxas(bhhone) = qxk(bhhone)
IF brw = 4 THEN qxbs(bhhone) = qxl(bhhone)
IF brw = 5 THEN qxas(bhhone) = qxm(bhhone)
IF brw = 5 THEN qxbs(bhhone) = qxn(bhhone)
IF brw = 6 THEN qxas(bhhone) = qxo(bhhone)
IF brw = 6 THEN qxbs(bhhone) = qxp(bhhone)
IF brw = 7 THEN qxas(bhhone) = qxq(bhhone)
IF brw = 7 THEN qxbs(bhhone) = qxr(bhhone)
IF brw = 8 THEN qxas(bhhone) = qxs(bhhone)
IF brw = 8 THEN qxbs(bhhone) = qxt(bhhone)
IF brw = 9 THEN qxas(bhhone) = qxu(bhhone)
IF brw = 9 THEN qxbs(bhhone) = qxv(bhhone)
brws = 0

DO
xx = INT(RND * (6 - brlgen)) + (1 - brlgen)
yy = INT(RND * (9 - brlgen)) + (9 - brlgen)
IF brws = 0 THEN qxcs(bhhone) = qxas(bhhone) + xx + propo
IF brws = 0 THEN qxds(bhhone) = qxbs(bhhone) + yy
IF brws = 1 THEN qxes(bhhone) = qxcs(bhhone) + xx + propo
IF brws = 1 THEN qxfs(bhhone) = qxds(bhhone) + yy
IF brws = 2 THEN qxgs(bhhone) = qxes(bhhone) + xx + propo
IF brws = 2 THEN qxhs(bhhone) = qxfs(bhhone) + yy
brws = brws + 1
IF brws = 3 THEN GOTO 391311
LOOP
2361312
IF brw = 0 THEN qxast(bhhone) = qxc(bhhone)
IF brw = 0 THEN qxbst(bhhone) = qxd(bhhone)
IF brw = 1 THEN qxast(bhhone) = qxe(bhhone)
IF brw = 1 THEN qxbst(bhhone) = qxf(bhhone)
IF brw = 2 THEN qxast(bhhone) = qxg(bhhone)
IF brw = 2 THEN qxbst(bhhone) = qxh(bhhone)
IF brw = 3 THEN qxast(bhhone) = qxi(bhhone)
IF brw = 3 THEN qxbst(bhhone) = qxj(bhhone)
IF brw = 4 THEN qxast(bhhone) = qxk(bhhone)
IF brw = 4 THEN qxbst(bhhone) = qxl(bhhone)
IF brw = 5 THEN qxast(bhhone) = qxm(bhhone)
IF brw = 5 THEN qxbst(bhhone) = qxn(bhhone)
IF brw = 6 THEN qxast(bhhone) = qxo(bhhone)
IF brw = 6 THEN qxbst(bhhone) = qxp(bhhone)
IF brw = 7 THEN qxast(bhhone) = qxq(bhhone)
IF brw = 7 THEN qxbst(bhhone) = qxr(bhhone)
IF brw = 8 THEN qxast(bhhone) = qxs(bhhone)
IF brw = 8 THEN qxbst(bhhone) = qxt(bhhone)
IF brw = 9 THEN qxast(bhhone) = qxu(bhhone)
IF brw = 9 THEN qxbst(bhhone) = qxv(bhhone)
brws = 0

DO
xx = INT(RND * (6 - brlgen)) + (1 - brlgen)
yy = INT(RND * (9 - brlgen)) + (9 - brlgen)
IF brws = 0 THEN qxcst(bhhone) = qxast(bhhone) + xx + propo
IF brws = 0 THEN qxdst(bhhone) = qxbst(bhhone) + yy
IF brws = 1 THEN qxest(bhhone) = qxcst(bhhone) + xx + propo
IF brws = 1 THEN qxfst(bhhone) = qxdst(bhhone) + yy
IF brws = 2 THEN qxgst(bhhone) = qxest(bhhone) + xx + propo
IF brws = 2 THEN qxhst(bhhone) = qxfst(bhhone) + yy
brws = brws + 1
IF brws = 3 THEN GOTO 391312
LOOP
                     '************************
236132                                     'BRANCH TWO SUB
'set starting point
IF w = 0 THEN qqxa(bhhtwo) = a
IF w = 0 THEN qqxb(bhhtwo) = B
IF w = 1 THEN qqxa(bhhtwo) = c
IF w = 1 THEN qqxb(bhhtwo) = d
IF w = 2 THEN qqxa(bhhtwo) = e
IF w = 2 THEN qqxb(bhhtwo) = f
IF w = 3 THEN qqxa(bhhtwo) = g
IF w = 3 THEN qqxb(bhhtwo) = h
IF w = 4 THEN qqxa(bhhtwo) = i
IF w = 4 THEN qqxb(bhhtwo) = j
IF w = 5 THEN qqxa(bhhtwo) = k
IF w = 5 THEN qqxb(bhhtwo) = l
IF w = 6 THEN qqxa(bhhtwo) = m
IF w = 6 THEN qqxb(bhhtwo) = n
IF w = 7 THEN qqxa(bhhtwo) = o
IF w = 7 THEN qqxb(bhhtwo) = p
IF w = 8 THEN qqxa(bhhtwo) = ab
IF w = 8 THEN qqxb(bhhtwo) = ac
IF w = 9 THEN qqxa(bhhtwo) = ad
IF w = 9 THEN qqxb(bhhtwo) = ae
IF w = 10 THEN qqxa(bhhtwo) = af
IF w = 10 THEN qqxb(bhhtwo) = ag
IF w = 11 THEN qqxa(bhhtwo) = ah
IF w = 11 THEN qqxb(bhhtwo) = ai
IF w = 12 THEN qqxa(bhhtwo) = aj
IF w = 12 THEN qqxb(bhhtwo) = ak
IF w = 13 THEN qqxa(bhhtwo) = al
IF w = 13 THEN qqxb(bhhtwo) = am
IF w = 14 THEN qqxa(bhhtwo) = an
IF w = 14 THEN qqxb(bhhtwo) = ao
IF w = 15 THEN qqxa(bhhtwo) = ap
IF w = 15 THEN qqxb(bhhtwo) = aq
IF w = 16 THEN qqxa(bhhtwo) = ar
IF w = 16 THEN qqxb(bhhtwo) = xax
IF w = 17 THEN qqxa(bhhtwo) = at
IF w = 17 THEN qqxb(bhhtwo) = au
IF w = 18 THEN qqxa(bhhtwo) = av
IF w = 18 THEN qqxb(bhhtwo) = aw
IF w = 19 THEN qqxa(bhhtwo) = ax
IF w = 19 THEN qqxb(bhhtwo) = ay
IF w = 20 THEN qqxa(bhhtwo) = az
IF w = 20 THEN qqxb(bhhtwo) = ba
IF w = 21 THEN qqxa(bhhtwo) = bb
IF w = 21 THEN qqxb(bhhtwo) = bc
IF w = 22 THEN qqxa(bhhtwo) = bd
IF w = 22 THEN qqxb(bhhtwo) = be
IF w = 23 THEN qqxa(bhhtwo) = BF
IF w = 23 THEN qqxb(bhhtwo) = bg
IF w = 24 THEN qqxa(bhhtwo) = bh
IF w = 24 THEN qqxb(bhhtwo) = bi

'determine general propogation direction
bdirt = INT(RND * 2) + 1
bronefactort = -1
redun = redun + bronefactort
IF ABS(redun) > 2 THEN bronefactort = -bronefactort
IF ABS(redun) > 2 THEN redun = 0

IF bdirt = 1 THEN bronefactort = 1
propot = (brlength * bronefactort)
bronesubonet = 0
bronesubtwot = 0
brwt = 0
DO
xxt = INT(RND * (5 - brlgen)) + (5 - brlgen)
yyt = INT(RND * (14 - brlgen)) + (3 - brlgen)
xxt = xxt * bronefactort
IF brwt = 0 THEN qqxc(bhhtwo) = qqxa(bhhtwo) + xxt + propot
IF brwt = 0 THEN qqxd(bhhtwo) = qqxb(bhhtwo) + yyt
IF brwt = 1 THEN qqxe(bhhtwo) = qqxc(bhhtwo) + xxt + propot
IF brwt = 1 THEN qqxf(bhhtwo) = qqxd(bhhtwo) + yyt
IF brwt = 2 THEN qqxg(bhhtwo) = qqxe(bhhtwo) + xxt + propot
IF brwt = 2 THEN qqxh(bhhtwo) = qqxf(bhhtwo) + yyt
IF brwt = 3 THEN qqxi(bhhtwo) = qqxg(bhhtwo) + xxt + propot
IF brwt = 3 THEN qqxj(bhhtwo) = qqxh(bhhtwo) + yyt
IF brwt = 4 THEN qqxk(bhhtwo) = qqxi(bhhtwo) + xxt + propot
IF brwt = 4 THEN qqxl(bhhtwo) = qqxj(bhhtwo) + yyt
IF brwt = 5 THEN qqxm(bhhtwo) = qqxk(bhhtwo) + xxt + propot
IF brwt = 5 THEN qqxn(bhhtwo) = qqxl(bhhtwo) + yyt
IF brwt = 6 THEN qqxo(bhhtwo) = qqxm(bhhtwo) + xxt + propot
IF brwt = 6 THEN qqxp(bhhtwo) = qqxn(bhhtwo) + yyt
IF brwt = 7 THEN qqxq(bhhtwo) = qqxo(bhhtwo) + xxt + propot
IF brwt = 7 THEN qqxr(bhhtwo) = qqxp(bhhtwo) + yyt
IF brwt = 8 THEN qqxs(bhhtwo) = qqxq(bhhtwo) + xxt + propot
IF brwt = 8 THEN qqxt(bhhtwo) = qqxr(bhhtwo) + yyt
IF brwt = 9 THEN qqxu(bhhtwo) = qqxs(bhhtwo) + xxt + propot
IF brwt = 9 THEN qqxv(bhhtwo) = qqxt(bhhtwo) + yyt
IF brwt = 10 THEN qqxw(bhhtwo) = qqxu(bhhtwo) + xxt + propot
IF brwt = 10 THEN qqxx(bhhtwo) = qqxv(bhhtwo) + yyt
IF bronesubonet = 3 THEN GOTO 2361321
391321
IF bronesubtwot = 5 THEN GOTO 2361322
391322
bronesubonet = bronesubonet + 1
bronesubtwot = bronesubtwot + 1

brwt = brwt + 1
IF brwt = 11 THEN GOTO 12632
LOOP
12632
bhhtwo = bhhtwo + 1
GOTO 39132
2361321 'first branch sub branch
IF brwt = 0 THEN qqxas(bhhtwo) = qqxc(bhhtwo)
IF brwt = 0 THEN qqxbs(bhhtwo) = qqxd(bhhtwo)
IF brwt = 1 THEN qqxas(bhhtwo) = qqxe(bhhtwo)
IF brwt = 1 THEN qqxbs(bhhtwo) = qqxf(bhhtwo)
IF brwt = 2 THEN qqxas(bhhtwo) = qqxg(bhhtwo)
IF brwt = 2 THEN qqxbs(bhhtwo) = qqxh(bhhtwo)
IF brwt = 3 THEN qqxas(bhhtwo) = qqxi(bhhtwo)
IF brwt = 3 THEN qqxbs(bhhtwo) = qqxj(bhhtwo)
IF brwt = 4 THEN qqxas(bhhtwo) = qqxk(bhhtwo)
IF brwt = 4 THEN qqxbs(bhhtwo) = qqxl(bhhtwo)
IF brwt = 5 THEN qqxas(bhhtwo) = qqxm(bhhtwo)
IF brwt = 5 THEN qqxbs(bhhtwo) = qqxn(bhhtwo)
IF brwt = 6 THEN qqxas(bhhtwo) = qqxo(bhhtwo)
IF brwt = 6 THEN qqxbs(bhhtwo) = qqxp(bhhtwo)
IF brwt = 7 THEN qqxas(bhhtwo) = qqxq(bhhtwo)
IF brwt = 7 THEN qqxbs(bhhtwo) = qqxr(bhhtwo)
IF brwt = 8 THEN qqxas(bhhtwo) = qqxs(bhhtwo)
IF brwt = 8 THEN qqxbs(bhhtwo) = qqxt(bhhtwo)
IF brwt = 9 THEN qqxas(bhhtwo) = qqxu(bhhtwo)
IF brwt = 9 THEN qqxbs(bhhtwo) = qqxv(bhhtwo)
brwst = 0

DO
xxt = INT(RND * (6 - brlgen)) + (1 - brlgen)
yyt = INT(RND * (9 - brlgen)) + (9 - brlgen)
IF brwst = 0 THEN qqxcs(bhhtwo) = qqxas(bhhtwo) + xxt + propot
IF brwst = 0 THEN qqxds(bhhtwo) = qqxbs(bhhtwo) + yyt
IF brwst = 1 THEN qqxes(bhhtwo) = qqxcs(bhhtwo) + xxt + propot
IF brwst = 1 THEN qqxfs(bhhtwo) = qqxds(bhhtwo) + yyt
IF brwst = 2 THEN qqxgs(bhhtwo) = qqxes(bhhtwo) + xxt + propot
IF brwst = 2 THEN qqxhs(bhhtwo) = qqxfs(bhhtwo) + yyt
brwst = brwst + 1
IF brwst = 3 THEN GOTO 391321
LOOP
2361322
IF brwt = 0 THEN qqxast(bhhtwo) = qqxc(bhhtwo)
IF brwt = 0 THEN qqxbst(bhhtwo) = qqxd(bhhtwo)
IF brwt = 1 THEN qqxast(bhhtwo) = qqxe(bhhtwo)
IF brwt = 1 THEN qqxbst(bhhtwo) = qqxf(bhhtwo)
IF brwt = 2 THEN qqxast(bhhtwo) = qqxg(bhhtwo)
IF brwt = 2 THEN qqxbst(bhhtwo) = qqxh(bhhtwo)
IF brwt = 3 THEN qqxast(bhhtwo) = qqxi(bhhtwo)
IF brwt = 3 THEN qqxbst(bhhtwo) = qqxj(bhhtwo)
IF brwt = 4 THEN qqxast(bhhtwo) = qqxk(bhhtwo)
IF brwt = 4 THEN qqxbst(bhhtwo) = qqxl(bhhtwo)
IF brwt = 5 THEN qqxast(bhhtwo) = qqxm(bhhtwo)
IF brwt = 5 THEN qqxbst(bhhtwo) = qqxn(bhhtwo)
IF brwt = 6 THEN qqxast(bhhtwo) = qqxo(bhhtwo)
IF brwt = 6 THEN qqxbst(bhhtwo) = qqxp(bhhtwo)
IF brwt = 7 THEN qqxast(bhhtwo) = qqxq(bhhtwo)
IF brwt = 7 THEN qqxbst(bhhtwo) = qqxr(bhhtwo)
IF brwt = 8 THEN qqxast(bhhtwo) = qqxs(bhhtwo)
IF brwt = 8 THEN qqxbst(bhhtwo) = qqxt(bhhtwo)
IF brwt = 9 THEN qqxast(bhhtwo) = qqxu(bhhtwo)
IF brwt = 9 THEN qqxbst(bhhtwo) = qqxv(bhhtwo)
brwst = 0

DO
xxt = INT(RND * (6 - brlgen)) + (1 - brlgen)
yyt = INT(RND * (9 - brlgen)) + (9 - brlgen)
IF brwst = 0 THEN qqxcst(bhhtwo) = qqxast(bhhtwo) + xxt + propot
IF brwst = 0 THEN qqxdst(bhhtwo) = qqxbst(bhhtwo) + yyt
IF brwst = 1 THEN qqxest(bhhtwo) = qqxcst(bhhtwo) + xxt + propot
IF brwst = 1 THEN qqxfst(bhhtwo) = qqxdst(bhhtwo) + yyt
IF brwst = 2 THEN qqxgst(bhhtwo) = qqxest(bhhtwo) + xxt + propot
IF brwst = 2 THEN qqxhst(bhhtwo) = qqxfst(bhhtwo) + yyt
brwst = brwst + 1
IF brwst = 3 THEN GOTO 391322
LOOP

SUB BranchOne
END SUB

SUB BranchThree
END SUB

SUB BranchTwo
END SUB

SUB EndGame
CLS
END
END SUB

SUB gap
FOR gp = 1 TO 3000
NEXT gp
END SUB

SUB Intracloud
LINE (0, 0)-(640, 309), 0, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF

LINE (0, 320)-(640, 400), 8, BF
times = 0
DO
qw = INT(RND * 100) + 1        '********MUST HAVE GRAPHICS MONITOR TO RUN***
LET qa = 0                     '********MUST HAVE GRAPHICS MONITOR TO RUN***
times = times + 1
IF times > 1 THEN GOTO 90210
DO                            '********MUST HAVE GRAPHICS MONITOR TO RUN***
qy = INT(RND * 10) + 1         '********MUST HAVE GRAPHICS MONITOR TO RUN***
qx = INT(RND * 10) + 1         '********MUST HAVE GRAPHICS MONITOR TO RUN***
qm = INT(RND * 4) + 1          '********MUST HAVE GRAPHICS MONITOR TO RUN***
LET qx = qx - 5 + qm - 2         '********MUST HAVE GRAPHICS MONITOR TO RUN***
LINE (qa, qw)-(qa + qy, qw + qx) '********MUST HAVE GRAPHICS MONITOR TO RUN***

LINE (qa + 2, qw)-(qa + qy + 2, qw + qx)
qs = INT(RND * 10) + 1
IF qs < 2 THEN GOSUB 30
ender = INT(RND * 500) + 500
LET qw = qw + qx
LET qa = qa + qy
IF qa > ender THEN GOTO 10
LOOP
10
qk = INT(RND * 10) + 1
IF qk < 3 THEN GOSUB 23
LOOP
30 LET qf = 0
LET qg = qw + qx
LET qn = qa + qy
neg = INT(RND * 2) + 1
ffg = 1
IF neg = 1 THEN ffg = -1

DO
LET qf = qf + 1
ql = INT(RND * 14) + 1
qh = INT(RND * 10) + 1
qz = INT(RND * 10) + 1
qz = qz * ffg
qe = INT(RND * 10) + 1
qe = qe * ffg
LET ql = ql + qe
LINE (qn, qg)-(qn + qh, qg + ql)
LET qg = qg + ql
LET qn = qn + qh
IF qf > 7 THEN RETURN
LOOP
23 LET qr = 0
LET qa = 199
LET qw = qw + 1
qB = INT(RND * 10) + 1
DO
IF qr = 0 THEN GOSUB 44
IF qr = 1 THEN LET qc = 3
IF qr = 2 THEN GOSUB 44
IF qr = 3 THEN LET qc = 0
LET qr = qr + 1
IF qr = 4 THEN RETURN
LOOP
44 IF qB > 5 THEN LET qc = 1
IF qB <= 5 THEN LET qc = 2
RETURN
9 LET qt = 0
DO
LET qt = qt + 1
IF qt = 100 THEN RETURN
GOTO 90210
LOOP
90210





LINE (0, 0)-(640, 309), 0, BF
LINE (0, 310)-(499, 319), 0, BF
LINE (20, 319)-(300, 319), 8
LINE (70, 318)-(250, 318), 8
LINE (120, 317)-(200, 317), 8
LINE (140, 316)-(180, 316), 8

LINE (501, 310)-(640, 319), 0, BF

LINE (0, 320)-(640, 400), 8, BF

END SUB

SUB Intro
MyName
VersionNum
B = 0
a = 320
g = 0
l = 0
s = 10
COLOR 15
DO
e = INT(RND * 10) - 5
c = a + e
d = B + 5
LINE (a, B)-(c, d), 15
LINE (a + 1, B)-(c + 1, d), 15
LINE (a + 2, B)-(c + 2, d), 15
g = g + 1
IF g = 5 THEN LINE (a, B)-(a - 40, B + 60)
IF g = 5 THEN LINE (a - 40, B + 60)-(a - 60, B + 70)
IF g = 5 THEN LINE (a - 40, B + 60)-(a - 42, B + 90)
IF g = 15 THEN LINE (a, B)-(a + 40, B + 60)
IF g = 15 THEN LINE (a + 40, B + 60)-(a + 60, B + 70)
IF g = 15 THEN LINE (a + 40, B + 60)-(a + 42, B + 90)
IF g = 20 THEN LINE (a, B)-(a - 30, B + 60)
IF g = 20 THEN LINE (a - 30, B + 60)-(a - 60, B + 70)
IF g = 20 THEN LINE (a - 30, B + 60)-(a - 42, B + 90)
IF g = 25 THEN LINE (a, B)-(a + 40, B + 60)
IF g = 25 THEN LINE (a + 40, B + 60)-(a + 60, B + 70)
IF g = 25 THEN LINE (a + 40, B + 60)-(a + 42, B + 90)
IF g = 45 THEN LINE (a, B)-(a + 40, B + 60)
IF g = 45 THEN LINE (a + 40, B + 60)-(a + 60, B + 70)
IF g = 45 THEN LINE (a + 40, B + 60)-(a + 42, B + 90)
IF g = 60 THEN LINE (a, B)-(a - 40, B + 60)
IF g = 60 THEN LINE (a - 40, B + 60)-(a - 60, B + 70)
IF g = 60 THEN LINE (a - 40, B + 60)-(a - 42, B + 90)


B = d
a = c
IF B > 400 THEN GOTO 90217
LOOP
90217
LINE (120, 320)-(530, 200), 0, BF
LINE (120, 320)-(530, 200), 15, B
rc = 8
m = 120
LINE (10 + m, 300)-(70 + m, 310), rc, BF    'S
LINE (70 + m, 300)-(50 + m, 260), rc, BF
LINE (70 + m, 260)-(10 + m, 250), rc, BF
LINE (10 + m, 250)-(30 + m, 220), rc, BF
LINE (10 + m, 220)-(70 + m, 210), rc, BF
LINE (80 + m, 210)-(140 + m, 310), rc, BF           'T
LINE (80 + m, 220)-(140 + m, 310), 0, BF
LINE (100 + m, 220)-(120 + m, 310), rc, BF
LET z = m + 70
LINE (80 + z, 210)-(140 + z, 310), rc, BF     'O
LINE (100 + z, 220)-(120 + z, 300), 0, BF
LET z = m + 140
LINE (80 + z, 210)-(140 + z, 310), rc, BF     'R
LINE (100 + z, 220)-(120 + z, 310), 0, BF
LINE (100 + z, 260)-(140 + z, 310), 0, BF
LINE (100 + z, 250)-(140 + z, 260), rc, BF
FOR r = 1 TO 20
LINE (90 + z + r, 260)-(130 + z + r, 310), rc
NEXT r
LET z = m + 220                                 'M
LINE (80 + z, 210)-(100 + z, 310), rc, BF
LINE (160 + z, 210)-(180 + z, 310), rc, BF
FOR m = 1 TO 20
LINE (80 + z + m, 210)-(120 + z + m, 270), rc
LINE (160 + z + m, 210)-(120 + z + m, 270), rc
NEXT m

COLOR 15
DO
w = INT(RND * 10) + 1
f = 0
5505
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
rc = 7
TitleStorm
COLOR 7
LOCATE 12, 1
PRINT "PRESS 'S' TO START"
t = 0
231
t = t + 1
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
IF t < 200 THEN GOTO 231
rc = 8
TitleStorm
COLOR 8
LOCATE 12, 1
PRINT "PRESS 'S' TO START"
l = 0
566
l = l + 1
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
IF l < 400 THEN GOTO 566
f = f + 1
IF f < w THEN GOTO 5505

w = INT(RND * 3) + 1
f = 0
5506
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
rc = 7
TitleStorm
COLOR 7
LOCATE 12, 1
PRINT "PRESS 'S' TO START"
t = 0
631
t = t + 1
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
IF t < 200 THEN GOTO 631
rc = 8
TitleStorm
COLOR 8
LOCATE 12, 1
PRINT "PRESS 'S' TO START"
l = 0
5666
l = l + 1
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
IF l < 140 THEN GOTO 5666
f = f + 1
IF f < w THEN GOTO 5506
g = INT(RND * 100) + 300
f = 0

4049
f = f + 1
q$ = INKEY$
IF q$ = "s" THEN GOTO 87102
IF f < g THEN GOTO 4049
LOOP


87102
Verse
END SUB

SUB MyName
LOCATE 1, 2
COLOR 14
PRINT "Î  by Daniel Robinson 1994 Î"
END SUB

SUB Quitprg

55112
CLS
INPUT "Do you want to quit the program now? (Enter 'Y' or 'N'): ", ender$
tal = 0
IF ender$ = "Y" THEN tal = 1
IF ender$ = "N" THEN tal = 1
IF tal = 0 THEN GOTO 55112
IF ender$ = "Y" THEN EndGame
IF ender$ = "N" THEN MENU = 1
END SUB

SUB RedrawTower
tcolor = 8
IF towerhit = 1 THEN tcolor = 7

LINE (500, 310)-(500, 319), tcolor
PSET (500, 310), 4
PSET (500, 314), 4







END SUB

SUB Stormdelay
DUR = INT(RND * TME) + THRS
IF DUR < 2 THEN DUR = 2
FOR DEL = 1 TO DUR

NEXT DEL
IF TME <= 0 THEN INTENSE = 0
IF INTENSE = 1 THEN LET TME = TME - 1000
IF INTENSE = 0 THEN LET TME = TME + 1000
IF INTENSE = 1 THEN LET THRS = THRS - 100
IF INTENSE = 0 THEN LET THRS = THRS + 100

STORMGONE = 0
IF TME >= 100000 THEN STORMGONE = 1

END SUB

SUB Thunder
IF SOUNDON <> 1 THEN GOTO 55501

IF closeone <> 1 THEN GOTO 33101
crack = 5000
min = 500
DO
SOUND crack, .1
SOUND 10000, .2
LET crack = crack - min
LET min = min + 10
IF crack < 0 THEN GOTO 55501
LOOP
33101
dura = INT(RND * 3)
DO
dura = dura - 1
thund = INT(RND * 7) + 1
IF thund = 1 THEN PLAY "mbt160l25o0bacgfedc "
IF thund = 2 THEN PLAY "mbt160l25o0bcgfedc "
IF thund = 3 THEN PLAY "mbt160l25o0bgfedc "
IF thund = 4 THEN PLAY "mbt160l25o0bfedc "
IF thund = 5 THEN PLAY "mbt160l25o0bedc "
IF thund = 6 THEN PLAY "mbt160l25o0bdc "
IF thund = 7 THEN PLAY "mbt160l25o0bc "
IF dura < 0 THEN GOTO 6612
LOOP
6612 PLAY "mbt160l25o0bacgfedc "
55501

END SUB

SUB TitleStorm
m = 120
LINE (10 + m, 300)-(70 + m, 310), rc, BF    'S
LINE (70 + m, 300)-(50 + m, 260), rc, BF
LINE (70 + m, 260)-(10 + m, 250), rc, BF
LINE (10 + m, 250)-(30 + m, 220), rc, BF
LINE (10 + m, 220)-(70 + m, 210), rc, BF
LINE (80 + m, 210)-(140 + m, 310), rc, BF           'T
LINE (80 + m, 220)-(140 + m, 310), 0, BF
LINE (100 + m, 220)-(120 + m, 310), rc, BF
LET z = m + 70
LINE (80 + z, 210)-(140 + z, 310), rc, BF     'O
LINE (100 + z, 220)-(120 + z, 300), 0, BF
LET z = m + 140
LINE (80 + z, 210)-(140 + z, 310), rc, BF     'R
LINE (100 + z, 220)-(120 + z, 310), 0, BF
LINE (100 + z, 260)-(140 + z, 310), 0, BF
LINE (100 + z, 250)-(140 + z, 260), rc, BF
FOR r = 1 TO 20
LINE (90 + z + r, 260)-(130 + z + r, 310), rc
NEXT r
LET z = m + 220                                 'M
LINE (80 + z, 210)-(100 + z, 310), rc, BF
LINE (160 + z, 210)-(180 + z, 310), rc, BF
FOR m = 1 TO 20
LINE (80 + z + m, 210)-(120 + z + m, 270), rc
LINE (160 + z + m, 210)-(120 + z + m, 270), rc
NEXT m

END SUB

SUB Verse
CLS
COLOR 3
LOCATE 12, 20
PRINT "F";
gap
PRINT "o";
gap
PRINT "r";
gap
PRINT " ";
gap
PRINT "G";
gap
PRINT "o";
gap
PRINT "d";
gap
PRINT " ";
gap
PRINT "s";
gap
PRINT "o";
gap
PRINT " ";
gap
PRINT "l";
gap
PRINT "o";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT "d";
gap
PRINT " ";
gap
PRINT "t";
gap
PRINT "h";
gap
PRINT "e";
gap
PRINT " ";
gap
PRINT "w";
gap
PRINT "o";
gap
PRINT "r";
gap
PRINT "l";
gap
PRINT "d";
gap
LOCATE 13, 20
PRINT "t";
gap
PRINT "h";
gap
PRINT "a";
gap
PRINT "t";
gap
PRINT " ";
gap
PRINT "H";
gap
PRINT "e";
gap
PRINT " ";
gap
PRINT "g";
gap
PRINT "a";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT " ";
gap
PRINT "H";
gap
PRINT "i";
gap
PRINT "s";
gap
PRINT " ";
gap
PRINT "o";
gap
PRINT "n";
gap
PRINT "l";
gap
PRINT "y";
gap
PRINT " ";
gap
PRINT "s";
gap
PRINT "o";
gap
PRINT "n";
gap
PRINT ",";
gap
PRINT " ";
gap
PRINT "s";
gap
PRINT "o";
gap
PRINT " ";
gap
PRINT "t";
gap
PRINT "h";
gap
PRINT "a";
gap
PRINT "t";
gap
LOCATE 14, 20
PRINT "w";
gap
PRINT "h";
gap
PRINT "o";
gap
PRINT "e";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT "r";
gap
PRINT " ";
gap
PRINT "b";
gap
PRINT "e";
gap
PRINT "l";
gap
PRINT "i";
gap
PRINT "e";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT "s";
gap
PRINT " ";
gap
PRINT "i";
gap
PRINT "n";
gap
PRINT " ";
gap
PRINT "H";
gap
PRINT "i";
gap
PRINT "m";
gap
PRINT " ";
gap
PRINT "w";
gap
PRINT "i";
gap
PRINT "l";
gap
PRINT "l";
gap
PRINT " ";
gap
PRINT "n";
gap
PRINT "o";
gap
PRINT "t";
gap
LOCATE 15, 20
PRINT "p";
gap
PRINT "e";
gap
PRINT "r";
gap
PRINT "i";
gap
PRINT "s";
gap
PRINT "h";
gap
PRINT ",";
gap
PRINT " ";
gap
PRINT "b";
gap
PRINT "u";
gap
PRINT "t";
gap
PRINT " ";
gap
PRINT "h";
gap
PRINT "a";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT " ";
gap
PRINT "e";
gap
PRINT "v";
gap
PRINT "e";
gap
PRINT "r";
gap
PRINT "l";
gap
PRINT "a";
gap
PRINT "s";
gap
PRINT "t";
gap
PRINT "i";
gap
PRINT "n";
gap
PRINT "g";
gap
PRINT " ";
gap
PRINT "l";
gap
PRINT "i";
gap
PRINT "f";
gap
PRINT "e";
gap
PRINT ".";
gap
PRINT ".";
gap
PRINT ".";
gap
PRINT " ";
gap
SLEEP 1
COLOR 7
LOCATE 17, 30
PRINT "J";
gap
PRINT "o";
gap
PRINT "h";
gap
PRINT "n";
gap
PRINT " ";
gap
PRINT "3";
gap
PRINT ":";
gap
PRINT "1";
gap
PRINT "6";
SLEEP 2
FOR scrl = 1 TO 27
FOR dl = 1 TO 3
gap
NEXT dl
PRINT "                "
NEXT scrl
END SUB

SUB VersionNum
COLOR 2
LOCATE 1, 46
PRINT "Version 4.4   (June 1997)"
COLOR 4
LOCATE 12, 50
PRINT "A Lightning Simulator in"
LOCATE 13, 55
PRINT "Microsoft Qbasic"
COLOR 1
LOCATE 13, 1
PRINT "Press CTRL-BREAK to Stop"
END SUB


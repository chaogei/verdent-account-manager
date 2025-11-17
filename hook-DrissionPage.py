# PyInstaller hook for DrissionPage
# 这个文件帮助 PyInstaller 正确打包 DrissionPage 的所有依赖

from PyInstaller.utils.hooks import collect_all, collect_submodules, collect_data_files

# 收集 DrissionPage 的所有内容
datas, binaries, hiddenimports = collect_all('DrissionPage')

# 额外确保收集所有子模块
hiddenimports += collect_submodules('DrissionPage')

# 收集相关依赖库
hiddenimports += [
    'websocket',
    'websocket._core',
    'websocket._app',
    'websocket._exceptions',
    'websocket._socket',
    'websocket._http',
    'websocket._url',
    'websocket._utils',
    'tldextract',
    'lxml',
    'lxml.html',
    'lxml.etree',
    'cssselect',
    'urllib3',
    'requests',
    'certifi',
    'charset_normalizer',
    'idna',
]

# 确保包含 DrissionPage 的配置文件
datas += collect_data_files('DrissionPage')

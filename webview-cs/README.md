## Electronとの比較（すこしElectronに寄せる）

### Electron

main.js -> index.html -> app.js

**Main Process  (main.js)**

```js
const {app, BrowserWindow, ipcMain} = require('electron')
const createWindow = () => {
  const win = new BrowserWindow({ width: 800, height: 1500 })
  win.loadURL('https://github.com')
  win.loadFile('index.html');
};
app.once('ready', () => { createWindow(); });

// send
win.webContents.send('message', 'ping');
// receive
ipcMain.on('message', (event, arg) => { })

// two-way (Renderer to Main)
ipcMain.handle('some-name', async (event, someArgument) => {
  const result = await doSomeWork(someArgument)
  return result
})
```

**Renderer Process (index.html/app.js)**

```html
<body>
  <h1>Hello, world.</h1>
  <script src="app.js"></script>
</body>
```
```js
const { ipcRenderer } = require('electron');
```

```js
const {ipcRenderer} = require('electron')

// send
ipcRenderer.send('message', 'ping');
ipcRenderer.sendSync('message', 'ping');
ipcRenderer.postMessage ('port', { message: 'hello' }, [port1]);
// receive
ipcRenderer.on('message', (event, arg) => { })
ipcRenderer.addListener('message', (event, arg) => { })

// two-way (Renderer to Main)
ipcRenderer.invoke('some-name', someArgument).then((result) => { });
```

### WebView2

MainView.cs -> MainViewModel.cs -> MainModel.cs -> index.html -> app.js

**Main Process (MainModel.cs/MainViewModel.cs/MainView.cs)**

```cs
await webView2.EnsureCoreWebView2Async(null);
await webView2.CoreWebView2.ExecuteScriptAsync();
webView.CoreWebView2.Navigate(uri.ToString());

// send
webView2.CoreWebView2.PostWebMessageAsString("test");
// receive
webView2.CoreWebView2.WebMessageReceived += CoreWebView2_WebMessageReceived;

// two-way (Renderer to Main)
await webView2.CoreWebView2.AddScriptToExecuteOnDocumentCreatedAsync("myHostObject", obj);
```

**Renderer Process (index.html/app.js)**

```js
// send
window.chrome.webview.postMessage(json)
// receive
window.chrome.webview.addEventListener('message', (e) => {});

// two-way (Renderer to Main)
var result = await chrome.webview.hostObjects.myHostObject.GetString();
```



## イベントの取り扱い

破棄忘れするのでRxを使おう

```cs
public event Action X;
private void RaiseX() => X?.Invoke();

Action handler = () => Console.WriteLine("X");
X += handler;
for (int i = 0; i < 5; i++) RaiseX();
X -= handler;
for (int i = 0; i < 5; i++) RaiseX();
```

```cs
var mouseDown = Observable.FromEventPattern(this, "MouseDown");
var mouseDown = Observable.FromEvent(h => (s, e) => h(e),　h => this.MouseDown += h,　h => this.MouseDown -= h);
```



- [イベントの購読とその解除](https://ufcpp.net/study/csharp/MiscEventSubscribe.html)
- [Reactive Extensions入門 + メソッド早見解説表](https://neue.cc/2010/07/28_269.html)
- [Rx FromEvent再訪(と、コードスニペット)](https://neue.cc/2011/02/18_303.html)
- [INotifyPropertyChangedプロパティ実装方法まとめ](https://qiita.com/soi/items/d0c83a0cc3a4b23237ef)

## MVVMでの画面遷移

### prism

```xml
<Window x:Class="PrismApp.Views.MainWindow"
  ...
  prism:ViewModelLocator.AutoWireViewModel="True">
  <Grid>
    <ContentControl prism:RegionManager.RegionName="ContentRegion" />
  </Grid>
</Window>
```

```cs
// View
public partial class App {
  protected override Window CreateShell() {
    return Container.Resolve<MainWindow>();
  }
  protected override void RegisterTypes(IContainerRegistry containerRegistry) {
    containerRegistry.RegisterForNavigation<Views.ViewA>();
    containerRegistry.RegisterForNavigation<Views.ViewB>();
  }
}

// ViewModel
public class MainWindowViewModel : BindableBase {
  private readonly IRegionManager _regionManager;
  public MainWindowViewModel(IRegionManager regionManager) {
    _regionManager = regionManager;
    _regionManager.RegisterViewWithRegion("ContentRegion", typeof(Views.ViewA));
  }
}
```

### webview without Locator

```xml
<Window x:Class="WVVMSample.MainWindow"
  ...
  xmlns:wv2="clr-namespace:Microsoft.Web.WebView2.Wpf;assembly=Microsoft.Web.WebView2.Wpf"
  xmlns:ex="clr-namespace:WVVM">
  <Window.DataContext>
    <local:MainWindowViewModel />
  </Window.DataContext>
  <Grid>
    <wv2:WebView2 x:Name="webView">
      <i:Interaction.Behaviors>
        <ex:WebView2Behavior
          Locator="False"
          ExecuteScript="{Binding Path=ExecuteScript.Value, Mode=OneWayToSource}">
          <![CDATA[
            <div id="app">{{label}}</div>
          ]]>
        </ex:WebView2Behavior>
      </i:Interaction.Behaviors>
    </wv2:WebView2>
  </Grid>
</Window>
```
```cs
// ViewModel in cs
public class MainWindowViewModel : BindableBase {
  public MainWindowViewModel() {
    NavigationCompleted = new ReactiveCommand<CoreWebView2NavigationCompletedEventArgs>().WithSubscribe(async (e) =>{
      await ExecuteScript.Value("content.js");
    });
  }
}
```
```js
// ViewModel in js
const viewmodel = {
  setup() {
    const label = window.Vue.ref('null');
    return {label}
  }
}
const app = window.Vue.createApp(viewmodel).mount('#app'); 
```

### webview with Locator

- 条件
  - 1. C#ではないのでPrismのロケータ, コンテナは使用できない
  - 2. 外部ファイル参照を利用
  - 2. 共通オブジェクトはコンテナを介す

include_str!ほしくなる

```xml
...
  <ex:WebView2Behavior Locator="True">
    {Binding Navigate.Value}
  </ex:WebView2Behavior>
```
```cs
// View
public partial class App {
  protected override Window CreateShell() {
    return Container.Resolve<MainWindow>();
  }
  protected override void RegisterTypes(IContainerRegistry containerRegistry) {
    containerRegistry.RegisterForNavigation<Views.ViewA>();
    containerRegistry.RegisterForNavigation<Views.ViewB>();
  }
}

// ViewModel
public class MainWindowViewModel : BindableBase {
  public MainWindowViewModel() {
    Navigate.Value = "path";
  }
}
```


## Rx比較

```xml
<!-- V <- VM (OneTime) -->
<Label Content="{Binding Path=Label.Value, Mode=OneTime}">
<!-- V <- VM -->
<Label Content="{Binding Path=Label.Value, Mode=OneWay}">
<!-- V <-> VM -->
<Label Content="{Binding Path=Label.Value, Mode=TwoWay}">
<!-- V -> VM -->
<Label Content="{Binding Path=Label.Value, Mode=OneWayToSource}">

<!-- Default : Label = OneWay, TextBox = TwoWay -->

<!-- UpdateSource()メソッドを呼んだときのみ反映 -->
<TexBox Text="{Binding Path=Text.Value, UpdateSourceTrigger=Explicit}">
<!-- コントロールがフォーカスを失ったとき -->
<TexBox Text="{Binding Path=Text.Value, UpdateSourceTrigger=LostFocus}">
<!-- プロパティ値が変更されたら即座に反映 -->
<TexBox Text="{Binding Path=Text.Value, UpdateSourceTrigger=PropertyChanged}">
```

```cs
// V <- M
this.OneWay = model.ObserveProperty(x => x.Label).ToReadOnlyReactiveProperty();
// V <- VM/M
this.OneWay = model.ObserveProperty(x => x.Label).ToReactiveProperty();
// V <-> M
this.TwoWay = model.ToReactivePropertyAsSynchronized(x => x.Label);
// V/VM -> M
this.OneWayToSource = ReactiveProperty.FromObject(poco, x => x.Name);

// V <-> VM
this.TwoWay = new ReactiveProperty<string>("foo");
this.TwoWay = new ReactiveProperty<string>();
// V -> VM -> V
this.TwoWay = new ReactiveProperty<string>();
this.OneWay = this.TwoWay.Select(s => s != null ? s.ToUpper() : null).ToReactiveProperty();
// V -> (VM) -> V
this.TwoWay = new ReadOnlyReactiveProperty<string>();
this.OneWay = this.TwoWay.Select(s => s != null ? s.ToUpper() : null).ToReadOnlyReactiveProperty();

```

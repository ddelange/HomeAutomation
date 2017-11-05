constexpr const char* AUDIOSYS = R"delimiter(<!doctype html><html lang="en"><head><meta charset="utf-8"/><link rel="icon"type="image/png"href="assets/img/favicon.ico"><meta http-equiv="X-UA-Compatible"content="IE=edge,chrome=1"/><title>HomeVid</title><meta content='width=device-width,initial-scale=1.0,maximum-scale=1.0,user-scalable=0'name='viewport'/><meta name="viewport"content="width=device-width"/><link href="assets/css/bootstrap.min.css"rel="stylesheet"/><link href="assets/css/animate.min.css"rel="stylesheet"/><link href="assets/css/light-bootstrap-dashboard.css"rel="stylesheet"/><link href="http://maxcdn.bootstrapcdn.com/font-awesome/4.2.0/css/font-awesome.min.css"rel="stylesheet"><link href='http://fonts.googleapis.com/css?family=Roboto:400,700,300'rel='stylesheet'type='text/css'><link href="assets/css/pe-icon-7-stroke.css"rel="stylesheet"/><script src="https://cdn.plot.ly/plotly-latest.min.js"></script></head><body><div class="wrapper"><div class="sidebar"data-color="blue"data-image="assets/img/sidebar-5.jpg"><div class="sidebar-wrapper"><div class="logo"><a class="simple-text">ControlPanel</a></div><ul class="nav"><li class="active"><a href="dashboard.html"><i class="pe-7s-display2"></i><p>Dashboard</p></a></li><li><a href="user.html"><i class="pe-7s-umbrella"></i><p>TemperatureandHumidity</p></a></li><li><a href="table.html"><i class="pe-7s-sun"></i><p>Co2andLight</p></a></li><li><a href="table.html"><i class="pe-7s-leaf"></i><p>SoilHumidity</p></a></li><li><a href="typography.html"><i class="pe-7s-users"></i><p>Movement</p></a></li><li><a href="icons.html"><i class="pe-7s-graph2"></i><p>CustomGraph</p></a></li><li><a href="maps.html"><i class="pe-7s-edit"></i><p>RoomState</p></a></li><li><a href="maps.html"><i class="pe-7s-map-marker"></i><p>GPStrackers</p></a></li><li><a href="maps.html"><i class="pe-7s-music"></i><p>AudioSystem</p></a></li><li><a href="notifications.html"><i class="pe-7s-bell"></i><p>Notifications</p></a></li></ul></div></div><div class="main-panel"><nav class="navbar navbar-default navbar-fixed"><div class="container-fluid"><div class="navbar-header"><button type="button"class="navbar-toggle"data-toggle="collapse"data-target="#navigation-example-2"><span class="sr-only">Togglenavigation</span><span class="icon-bar"></span><span class="icon-bar"></span><span class="icon-bar"></span></button><a class="navbar-brand"href="#">Dashboard</a></div><div class="collapse navbar-collapse"><ul class="nav navbar-nav navbar-left"></ul><ul class="nav navbar-nav navbar-right"><li><a href=""><p>Account</p></a></li><li class="dropdown"><a href="#"class="dropdown-toggle"data-toggle="dropdown"><p>Quick-Actions<b class="caret"></b></p></a><ul class="dropdown-menu"><li><a href="#">Turnonlamps</a></li><li><a href="#">Turnofflamps</a></li><li class="divider"></li><li><a href="#">nightstate</a></li><li><a href="#">eveningstate</a></li><li><a href="#">defaultstate</a></li><li><a href="#">moviestate</a></li></ul></li><li><a href="#"><p>Logout</p></a></li><li class="separator hidden-lg hidden-md"></li></ul></div></div></nav><div class="content"><div class="container-fluid"><div class="row"><div class="col-md-13"><div class="card"><div class="header"><h4 class="title">ShortRoomState</h4><p class="category">Currentvalueofvaluesbelow</p
 </div><div class="content table-responsive table-full-width"><table class="table table-hover"><thead><th>Audiosystem</th><th>RoomState</th><th>Temperature</th><th>Humidity</th><th>CO2</th><th>Light</th><th>LastMovement</th></thead><tbody><tr><td>)delimiter";
constexpr const char* ROOMSTATE = R"delimiter(</td><td>)delimiter";
constexpr const char* TEMP = R"delimiter(</td><td>)delimiter";
constexpr const char* HUMID = R"delimiter(</td><td>)delimiter";
constexpr const char* CO2 = R"delimiter(</td><td>)delimiter";
constexpr const char* LIGHT = R"delimiter(</td><td>)delimiter";
constexpr const char* MOVEMENT = R"delimiter(</td><td>)delimiter";
constexpr const char* LAST = R"delimiter(</td></tr></tbody></table></div></div></div></div><div class="row"><div class="col-md-6"><div class="card"><div class="header"><h4 class="title">TestPlot</h4><p class="category">temperaturevshumidity</p></div><div class="content2"><div id="testPlot"></div></div><script>var trace1={x:[1,2,3,4],y:[10,15,13,17],type:'scatter',yaxis:'y1'};var trace2={x:[1,2,3,4],y:[16,5,11,9],type:'scatter',yaxis:'y2',};var layout={showlegend:true,legend:{"orientation":"h"},margin:{l:40,r:40,b:0,t:0,pad:0},yaxis:{title:'test1'},yaxis2:{overlaying:'y',side:'right',title:'humidity (percent)'}};var data=[trace1,trace2];Plotly.newPlot('testPlot',data,layout);window.addEventListener('resize',function(){var update={width:document.getElementById('testPlot').clientWidth,height:document.getElementById('testPlot').clientHeight};Plotly.relayout('testPlot',update);},true);</script></div></div><div class="col-md-6"><div class="card"><div class="header"><h4 class="title">Bathroom</h4><p class="category">last2hourstemperatureandhumidity</p></div><div class="content2"><div id="myDiv"></div><script src="https://deviousd.duckdns.org:8444/bathRoomJS.js"></script></div></div></div></div><div class="row"><div class="col-md-6"><div class="card"><div class="header"><h4 class="title">TestPlot2</h4><p class="category">temperature2vshumidity2</p></div><div class="content2"><div id="testPlot2"></div></div><script src="testScripts/testPlot2.js"></script></div></div></div></div></div></div></div></body><script src="assets/js/jquery-1.10.2.js"type="text/javascript"></script><script src="assets/js/bootstrap.min.js"type="text/javascript"></script><script src="assets/js/bootstrap-checkbox-radio-switch.js"></script><script src="assets/js/bootstrap-notify.js"></script><script type="text/javascript"src="https://maps.googleapis.com/maps/api/js?sensor=false"></script><script src="assets/js/light-bootstrap-dashboard.js"></script></html>)delimiter";

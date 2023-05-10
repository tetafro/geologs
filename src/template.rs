pub const TEMPLATE: &str = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="Content-Type" content="text/html; charset=utf-8">
    <title>Access Logs</title>
    <link rel="shortcut icon" href="favicon.png">
    <link rel="stylesheet" href="./static/tabler.min.css">
    <link rel="stylesheet" href="./static/jsvectormap.min.css">
</head>

<body>
    <div class="page">
        <header class="navbar navbar-expand-md navbar-light d-print-none">
            <div class="container-xl">
                <h1 class="navbar-brand">Access Logs</h1>
            </div>
        </header>
        <div class="page-wrapper">
            <div class="page-body">
                <div class="container-xl">
                    <div class="row row-deck row-cards">
                        <div class="col-lg-12">
                            <div class="btn-group w-100" role="group">
                                <input type="radio" class="btn-check" name="radio=period" id="radio-period-all">
                                <label for="radio-period-all" type="button" class="btn">All</label>
                                <input type="radio" class="btn-check" name="radio=period" id="radio-period-week">
                                <label for="radio-period-week" type="button" class="btn">Week</label>
                                <input type="radio" class="btn-check" name="radio=period" id="radio-period-month" checked="">
                                <label for="radio-period-month" type="button" class="btn">Month</label>
                                <input type="radio" class="btn-check" name="radio=period" id="radio-period-3months">
                                <label for="radio-period-3months" type="button" class="btn">3 Months</label>
                                <input type="radio" class="btn-check" name="radio=period" id="radio-period-year">
                                <label for="radio-period-year" type="button" class="btn">Year</label>
                            </div>
                        </div>
                        <div class="col-lg-12">
                            <div class="card">
                                <div class="card-body">
                                    <h3 class="card-title">Visitors by date</h3>
                                    <div id="visitors-by-date" class="chart-lg"></div>
                                </div>
                            </div>
                        </div>
                        <div class="col-lg-8">
                            <div class="card">
                                <div class="card-header">
                                    <h3 class="card-title">Visitors by country</h3>
                                </div>
                                <div class="card-body">
                                    <div class="ratio ratio-21x9">
                                        <div>
                                            <div id="map-world" class="w-100 h-100"></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="col-lg-4">
                            <div class="card">
                                <div class="card-header">
                                    <h3 class="card-title">Visitors by city</h3>
                                </div>
                                <table class="table card-table table-vcenter">
                                    <thead>
                                        <tr>
                                            <th>City</th>
                                            <th>Country</th>
                                            <th>Visitors</th>
                                        </tr>
                                    </thead>
                                    <tbody id="table-cities"></tbody>
                                </table>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>

<script src="./static/jsvectormap.js"></script>
<script src="./static/world.js"></script>
<script src="./static/tabler.js"></script>
<script src="./static/apexcharts.js"></script>

<script>
    var visitorsByDate;
    var worldMap;
    var dataLog = [
        {%- for line in lines %}
        {
            date: "{{ line.date }}",
            ip: "{{ line.ip }}",
            country: "{{ line.country }}",
            countryCode: "{{ line.country_code }}",
            city: "{{ line.city }}",
            lat: "{{ line.lat }}",
            lon: "{{ line.lon }}",
        },
        {%- endfor %}
    ];

    function filterByDate(data, period) {
        var periods = {
            "week": 7,
            "month": 30,
            "3months": 3 * 30,
            "year": 365,
            "all": 50 * 365,
        }
        var dateFrom = new Date();
        dateFrom.setDate(dateFrom.getDate() - periods[period]);
        return data.filter((line) => {
            return Date.parse(line.date) >= dateFrom;
        });
    }

    function groupByDates(data, period) {
        var filtered = filterByDate(data, period);
        var days = {};
        for (let i = 0; i < filtered.length; i++) {
            var line = filtered[i];
            if (!days[line.date]) {
                days[line.date] = new Set();
            }
            days[line.date].add(line.ip);
        }

        var grouped = [];
        for (var day in days) {
            grouped.push({x: day, y: days[day].size})
        }
        return grouped;
    }

    function groupByCities(data, period) {
        var filtered = filterByDate(data, period);
        var cities = {};
        for (let i = 0; i < filtered.length; i++) {
            var line = filtered[i];
            if (line.city == "") {
                continue;
            }
            if (!cities[line.city]) {
                cities[line.city] = {
                    count: new Set(),
                    country: line.country,
                    coords: [line.lat, line.lon],
                };
            }
            cities[line.city].count.add(line.ip);
        }

        var grouped = [];
        for (var city in cities) {
            grouped.push({
                name: city,
                country: cities[city].country,
                count: cities[city].count.size,
                coords: cities[city].coords,
            })
        }
        return grouped;
    }

    function groupByCountries(data, period) {
        var filtered = filterByDate(data, period);
        var countries = {};
        for (let i = 0; i < filtered.length; i++) {
            var line = filtered[i];
            if (line.countryCode == "") {
                continue;
            }
            if (!countries[line.countryCode]) {
                countries[line.countryCode] = new Set();
            }
            countries[line.countryCode].add(line.ip);
        }

        for (country in countries) {
            countries[country] = countries[country].size
        }
        return countries;
    }

    function updateTable(data, period) {
        var table = document.getElementById("table-cities");
        while (table.firstChild) {
            table.removeChild(table.firstChild);
        }

        var cities = groupByCities(data, period);
        cities.sort(function (a, b) {
            return a.count < b.count;
        })
        var limit = Math.min(cities.length, 8);
        for (let i = 0; i < limit; i++) {
            var row = table.insertRow();
            var nameCell = row.insertCell();
            var nameText = document.createTextNode(cities[i].name);
            nameCell.appendChild(nameText);
            var countryCell = row.insertCell();
            var countryText = document.createTextNode(cities[i].country);
            countryCell.appendChild(countryText);
            var countCell = row.insertCell();
            var countText = document.createTextNode(cities[i].count);
            countCell.appendChild(countText);
        }
    }

    function updateMap(data, period) {
        // Update contries colors
        var mapData = groupByCountries(data, period);
        for (country in groupByCountries(data, "all")) {
            // Fill other countries with zeros, otherwise they will keep
            // their previous color
            if (!mapData[country]) {
                mapData[country] = 0;
            }
        }
        worldMap.dataVisualization._values = mapData;
        worldMap.dataVisualization.setMinMaxValues(mapData);
        worldMap.dataVisualization.visualize();

        // Update markers
        worldMap.removeMarkers();
        worldMap.addMarkers(groupByCities(dataLog, period));
    }

    function updatePeriod(data, period) {
        visitorsByDate.updateSeries([{
            name: "Visitors",
            data: groupByDates(data, period)
        }]);
        updateTable(data, period);
        updateMap(data, period);
    }

    document.addEventListener("DOMContentLoaded", () => {
        var periods = ["all", "week", "month", "3months", "year"];
        var period = "all";
        for (let i = 0; i < periods.length; i++) {
            var btn = document.getElementById("radio-period-"+periods[i]);
            if (btn.checked) {
                period = periods[i];
                break;
            }
        }

        // Views by date panel
        visitorsByDate = new ApexCharts(document.getElementById("visitors-by-date"), {
            chart: {
                type: "bar",
                fontFamily: "inherit",
                height: 240,
                parentHeightOffset: 0,
                toolbar: {show: false},
                animations: {enabled: false},
            },
            plotOptions: {bar: {columnWidth: "50%"}},
            dataLabels: {enabled: false},
            fill: {opacity: 1},
            series: [{
                name: "Visitors",
                data: groupByDates(dataLog, period)
            }],
            grid: {
                padding: {
                    top: -20,
                    right: 0,
                    left: -4,
                    bottom: -4
                },
                strokeDashArray: 4,
                xaxis: {lines: {show: true}},
            },
            xaxis: {
                labels: {padding: 0},
                tooltip: {enabled: false},
                axisBorder: {show: false},
                type: "datetime",
            },
            yaxis: {labels: {padding: 4}},
            colors: ["#206bc4", "#79a6dc", "#bfe399"],
            legend: {show: false},
        });
        visitorsByDate.render();

        // World map
        worldMap = new jsVectorMap({
            selector: "#map-world",
            map: "world",
            backgroundColor: "transparent",
            regionStyle: {
                initial: {
                    fill: "#f8fafc",
                    stroke: "#e5e5e5",
                    strokeWidth: 0.5,
                }
            },
            zoomButtons: false,
            markers: groupByCities(dataLog, period),
            visualizeData: {
                scale: ["#f8fafc", "#206bc4"],
                values: groupByCountries(dataLog, period),
            },
        });
        window.addEventListener("resize", () => { worldMap.updateSize(); });

        // Visitors by city table
        updateTable(dataLog, period);
    });

    document.getElementById("radio-period-all").addEventListener(
        "click", () => { updatePeriod(dataLog, "all"); }
    );
    document.getElementById("radio-period-week").addEventListener(
        "click", () => { updatePeriod(dataLog, "week"); }
    );
    document.getElementById("radio-period-month").addEventListener(
        "click", () => { updatePeriod(dataLog, "month"); }
    );
    document.getElementById("radio-period-3months").addEventListener(
        "click", () => { updatePeriod(dataLog, "3months"); }
    );
    document.getElementById("radio-period-year").addEventListener(
        "click", () => { updatePeriod(dataLog, "year"); }
    );
</script>
</html>
"###;

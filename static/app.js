function load_scoreboard() {
    var scoreboard = $("#scoreboard");
    scoreboard.empty();
    $.ajax({
        url:'scoreboard',
        type:'GET',
        success:function(res){
            var scores = new Array();
            console.log(res);

            for (var team in res) {
                scores.push({name: team, score: res[team]});
            }

            scores.sort(function(a,b) { return a.score - b.score});

            for (var team_score of scores) {
                scoreboard.append("<tr><td>" + team_score.name + "</td><td>" + team_score.score + "</td></tr>")
            }
        },
        error:function(res){
            alert("Bad thing happend! " + res.statusText);
        }
    });
}

function submit_files() {
    var solution_form = document.forms["solution_files"];

    var reg_form = document.forms["registration"];
    var team_name = reg_form["Team Name"].value;
    var token = reg_form["Token"].value;

    var sol_a = solution_form["a"].files[0].text().then(function(d) {
        console.log("Got data for a");

        var sol = {
            "challenge": "Qual2020",
            "solutions": {
                "a": d
                }
        };

        $.ajax({
            url:'team/'+team_name+'/'+token+'/submit',
            type:'POST',
            data: JSON.stringify(sol),
            dataType: 'json',
            contentType: "application/json; charset=utf-8",
            success:function(res){
                console.log(res);
                alert("Successful submit");
            },
            error:function(res){
                console.log("Bad thing happend!");
                console.log(res);
            }
        });

    });
}

function list_teams() {
    $.ajax({
        url:'register_team',
        type:'POST',
        data: JSON.stringify(team),
        dataType: 'json',
        contentType: "application/json; charset=utf-8",
        success:function(res){
            token_as_hex = array_to_hex(res.token);
            console.log(token_as_hex);
            reg_form["Token"].value = token_as_hex;
        },
        error:function(res){
            alert("Bad thing happend! " + res.statusText);
        }
    });
}

function array_to_hex(array) {
  if (!array || array.length === 0) return '';
  var hex, i;
  var result = '';
  for (i = 0; i < array.length; i++) {
    hex = array[i].toString(16);
    if (hex.length === 1) {
      hex = '0' + hex;
    }
    result += hex;
  }
  return result;
};

function register_team() {
    var reg_form = document.forms["registration"];

    var team = {
        "name": reg_form["Team Name"].value,
        "participants": []
    };

    if (reg_form["Token"].value.length > 0) {
        alert("You already have a token");
    } else {
        console.log("Register: " + reg_form["Team Name"].value);
        $.ajax({
            url:'register_team',
            type:'POST',
            data: JSON.stringify(team),
            dataType: 'json',
            contentType: "application/json; charset=utf-8",
            success:function(res){
                console.log(res);
                if (res === "ErrorTeamExists") {
                    alert("This team already exists...");
                    return
                }
                token_as_hex = array_to_hex(res.token);
                console.log(token_as_hex);
                reg_form["Token"].value = token_as_hex;
            },
            error:function(res){
                alert("Bad thing happend! " + res.statusText);
            }
        });
    }

}

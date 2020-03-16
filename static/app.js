var submission_structure_per_challenge = {
    "qualification-2020": {
        "challenge" : {
            "Qualification": 2020
        },
        "files": ["a", "b", "c", "d", "e", "f"]
    },
    "qualification-2021": {
        "challenge" : {
            "Qualification": 2021
        },
        "files": ["e", "f"]
    },
}

$(document).ready( function() {
    for (var known_challenge in submission_structure_per_challenge) {
        $("#challenge-select").append("<option value=" + known_challenge + ">" + known_challenge + "</option>");
    };

    change_challenge()

    $("#challenge-select").change(change_challenge);
});

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

            scores.sort(function(a,b) { return b.score - a.score});

            for (var team_score of scores) {
                scoreboard.append("<tr><td>" + team_score.name + "</td><td>" + team_score.score + "</td></tr>")
            }
        },
        error:function(res){
            alert("Bad thing happend! " + res.statusText);
        }
    });
}


function change_challenge() {
    $("#solution-files").empty();
    for (var file_name of submission_structure_per_challenge[$("#challenge-select")[0].value].files) {
        $("#solution-files").append(
        "<label>" + file_name +
        "<input type=\"file\" name=" + file_name + ">" +
        "</label>"
        )
    };
}

function submit_files() {
    var solution_form = document.forms["solution-files"];

    var reg_form = document.forms["registration"];
    var team_name = reg_form["Team Name"].value;
    var token = reg_form["Token"].value;

    var sub_structure = submission_structure_per_challenge[$("#challenge-select")[0].value];

    var sol_array = new Array();

    for (var file_name of sub_structure.files) {
        var file_h = solution_form[file_name].files[0];
        if (file_h == null) {
            var p_text = null;
        } else {
            var p_text = file_h.text();
        }
        sol_array.push(p_text);
    }
    Promise.all(sol_array).then(function(fs) {
        var sol = {
            "challenge": sub_structure.challenge,
            "solutions": {}
        };

        for (var i=0; i<sub_structure.files.length; i++) {
            if (fs[i] != null) {
                sol.solutions[sub_structure.files[i]] = fs[i];
            }
        }

        console.log(sol);

        $.ajax({
            url:'team/'+team_name+'/'+token+'/submit',
            type:'POST',
            data: JSON.stringify(sol),
            dataType: 'json',
            contentType: "application/json; charset=utf-8",
            success:function(res){
                console.log("successful submit - scored " + res);
                alert("Successful submit - scored " + res);
            },
            error:function(jqxhr, status){
                alert("Submission failed: " + status)
                console.log(jqxhr);
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

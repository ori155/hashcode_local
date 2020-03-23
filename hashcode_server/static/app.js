var submission_structure_per_challenge = {
    "qualification-2020": {
        "challenge" : {
            "Qualification": 2020
        },
        "files": ["a", "b", "c", "d", "e", "f"],
        "pdf_file": "hashcode_2020_online_qualification_round.pdf",
        "in_files": "qualification_round_2020.in.zip",
        "scoreboard": "/scoreboard/qual2020"

    },
    "qualification-2016": {
        "challenge" : {
            "Qualification": 2016
        },
        "files": ["example", "busy_day", "mother_of_all_warehouses", "redundancy"],
        "pdf_file": "hashcode2016_qualification_task.pdf",
        "in_files": "qualification_round_2016.in.zip",
        "scoreboard": "/scoreboard/qual2016"
    },
}

var scoreboard_should_update = false;

function start_scoreboard_refresh() {
    scoreboard_should_update = true;
    function inner_refresh() {
        load_scoreboard();
        if (scoreboard_should_update) {
            window.setTimeout(inner_refresh, 5000);
        }
    }
    inner_refresh();
}

function stop_scoreboard_refresh() {
    scoreboard_should_update = false;
}

$(document).ready( function() {
    for (var known_challenge in submission_structure_per_challenge) {
        $("#challenge-select").append("<option value=" + known_challenge + ">" + known_challenge + "</option>");
    };

    change_challenge();
    $("#challenge-select").change(change_challenge);

    show_only('home');
    load_scoreboard();
});


function load_scoreboard() {
    var scoreboard = $("#scoreboard-table");
    var scoreboard_url = submission_structure_per_challenge[$("#challenge-select")[0].value].scoreboard;
    $.ajax({
        url: scoreboard_url,
        type:'GET',
        success:function(res){
            var scores = new Array();
            console.log(res);

            for (var team in res) {
                scores.push({name: team, score: res[team]});
            }

            scores.sort(function(a,b) { return b.score - a.score});

            scoreboard.empty();
            scoreboard.append("<thead><tr><th>#</th><th>Team Name</th><th>Total Score</th></tr><thead>");
            for (var i=0; i<scores.length; i++) {
                var team_score = scores[i];
                scoreboard.append("<tr><td>" + (i+1) + "</td><td>" + team_score.name + "</td><td>" + team_score.score + "</td></tr>");
            }
        },
        error:function(res){
            alert("Bad thing happend! " + res.statusText);
        }
    });
}


function show_only(pn) {
    if (pn === 'scoreboard') {
        load_scoreboard();
        window.setTimeout(
            start_scoreboard_refresh,
            1000);
    } else {
        stop_scoreboard_refresh();
    }
    $(".page[id!='" + pn + "']").hide();
    $("#" + pn).show();
}

function clear_submission_files() {
    $("#solution-files input[type='file']").val("");
}

function change_challenge() {
    var sol_files_form = $("#solution-files");
    var sub_structure = submission_structure_per_challenge[$("#challenge-select")[0].value];

    sol_files_form.empty();
    for (var file_name of sub_structure.files) {
        sol_files_form.append(
        "<div class='form-group'>" +
        "<label>" + file_name +
        "<input class='form-control-file' type='file' name=" + file_name + ">" +
        "</label>" +
        "</div>"
        )
    };


    sol_files_form.append(
        "<div class='btn-group' role='group' aria-label='submit-or-clear'>" +
        "<input class='btn btn-danger btn-lg' type='button' value='Clear' onclick='clear_submission_files()'>" +
        "<input class='btn btn-dark btn-lg' type='button' value='Score' onclick='submit_files(); show_only(\"my-submissions\")'>" +
        "</div>"
        );

    $("#download-links").empty();
    $("#download-links").append(
        "<a href='" + sub_structure.pdf_file + "' target='_blank'>Problem statement</a>"
    );

    $("#download-links").append(
        "<a href='" + sub_structure.in_files + "'>Input files</a>"
    );

    load_scoreboard();
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
            "team_name": team_name,
            "token": token,
            "solution": {
                "challenge": sub_structure.challenge,
                "solutions": {}
            }
        };

        for (var i=0; i<sub_structure.files.length; i++) {
            if (fs[i] != null) {
                sol.solution.solutions[sub_structure.files[i]] = fs[i];
            }
        }

        console.log(sol);

        $.ajax({
            url:'/submit',
            type:'POST',
            data: JSON.stringify(sol),
            dataType: 'json',
            contentType: "application/json; charset=utf-8",
            success:function(res){
                console.log("successful submit - scored " + res);
                var last_submissions_table = $("#last-submission");
                var currentdate = new Date();
                var datetime = currentdate.getDate() + "/"
                                + (currentdate.getMonth()+1)  + "/"
                                + currentdate.getFullYear() + " @ "
                                + currentdate.getHours() + ":"
                                + currentdate.getMinutes() + ":"
                                + currentdate.getSeconds();

                for (var in_file_name in res) {
                    last_submissions_table.append("<tr><td>" + datetime +"</td><td>" + in_file_name + "</td><td>" + res[in_file_name] + "</td></tr>");
                }
            },
            error:function(jqxhr, status){
                alert("Submission failed: " + jqxhr.responseText)
                console.log("Submission failed: " + status)
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

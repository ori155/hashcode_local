import re
from collections import namedtuple

Sub = namedtuple('Sub', 'hours_from_start team input_file score')
TotalScore = namedtuple('TotalScore', 'hours_from_start team total_score')

START_TIME = 9.0

def subs_from_log(d):
    for sl in re.findall(r"2020-03-25T([\d]+):([\d]+):([\d]+).*Team '([^']+)'.*scored ([\d]+).*InputFileName: ([_a-z]+)", d):
        # Add 2 hours to adjust time zone
        yield Sub(hours_from_start=((float(sl[0]) + 2) + float(sl[1])/60 + float(sl[2])/(60*60)) - START_TIME,
                team=sl[3],
                score=int(sl[4]),
                input_file=sl[5])

def total_score_over_time(team, subs):
    input_files = set(s.input_file for s in subs)
    max_per_input_file = {ifn: 0 for ifn in input_files}
    for s in subs:
        if s.team != team:
            continue
        if s.score > max_per_input_file[s.input_file]:
            max_per_input_file[s.input_file] = s.score
            yield TotalScore(hours_from_start=s.hours_from_start,
                    team=team,
                    total_score=sum(max_per_input_file.values()))

if __name__ == '__main__':
    import matplotlib.pyplot as plt

    d = open('log.txt', 'r').read()
    subs = list(subs_from_log(d))
    
    teams = set(s.team for s in subs if '12344' not in s.team)
    plt.figure(0)
    for team in teams:
        total_scores = list(total_score_over_time(team, subs))

        plt.figure(0)
        plt.plot([ts.hours_from_start for ts in total_scores], [ts.total_score for ts in total_scores], 'o', label=team)
        plt.plot([ts.hours_from_start for ts in total_scores], [ts.total_score for ts in total_scores], 'k--')

        plt.figure()
        plt.plot([ts.hours_from_start for ts in total_scores], [ts.total_score for ts in total_scores], '--')
        plt.plot([ts.hours_from_start for ts in total_scores], [ts.total_score for ts in total_scores], 'x')
        plt.title('Team "{}"'.format(team))
        plt.xlabel('Time Since start [hours]')
        plt.ylabel('Total Score [points]')
        plt.grid(True)
        plt.savefig('plots/'+team)
    
    plt.figure(0)
    plt.title('All teams')
    plt.xlabel('Time Since start [hours]')
    plt.ylabel('Total Score [points]')
    plt.legend(loc='upper left')
    plt.grid(True)
    plt.savefig('plots/all_teams')

    plt.show()
